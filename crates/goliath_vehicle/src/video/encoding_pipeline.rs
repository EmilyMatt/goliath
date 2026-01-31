use crate::error::GoliathVehicleResult;
use crate::video::PipelineWrapper;
use crate::video::rtp_pipeline::RTPPipeline;
use goliath_common::{GoliathGstAppsrc, GoliathGstPipeline, GoliathVideoError};
use gstreamer::ClockTime;
use gstreamer::prelude::{Cast, ElementExt, ElementExtManual, GstBinExtManual};
use gstreamer_app::gst;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[allow(dead_code)]
pub(crate) enum EncoderType {
    Software,
    V4L2,
    NVENC,
}

pub(crate) struct EncodingPipline {
    rtp_pipeline: Arc<dyn GoliathGstAppsrc>,
    pipeline: PipelineWrapper,
    appsrc: gstreamer_app::AppSrc,
    appsink: gstreamer_app::AppSink,
    started: AtomicBool,
    stopped: AtomicBool,
}

impl EncodingPipline {
    pub(crate) fn try_new(
        encoder_type: EncoderType,
        rtp_pipeline: Arc<RTPPipeline>,
    ) -> GoliathVehicleResult<Self> {
        let pipeline = gstreamer::Pipeline::builder()
            .name("EncodingPipeline")
            .async_handling(false)
            .latency(ClockTime::from_mseconds(0))
            .build();

        let appsrc = gstreamer_app::AppSrc::builder()
            .name("appsrc")
            .do_timestamp(true)
            .format(gstreamer::Format::Time)
            .build();

        let (encoder, capsfilter) = match encoder_type {
            EncoderType::V4L2 => (
                gstreamer::ElementFactory::make("v4l2h264enc")
                    .name("encoder")
                    .build()?,
                gstreamer::ElementFactory::make("capsfilter")
                    .name("caps_filter")
                    .property(
                        "caps",
                        gstreamer::Caps::builder("video/x-h264")
                            .field("profile", "main")
                            .field("level", "4")
                            .build(),
                    )
                    .build()?,
            ),
            EncoderType::NVENC => (
                gstreamer::ElementFactory::make("nvh264enc")
                    .name("encoder")
                    .property("aud", true)
                    .property("gop-size", 15)
                    .property("preset", "low-latency-hp")
                    .property("rc-mode", "cbr")
                    .property("zerolatency", true)
                    .build()?,
                gstreamer::ElementFactory::make("capsfilter")
                    .name("caps_filter")
                    .property(
                        "caps",
                        gstreamer::Caps::builder("video/x-h264")
                            .field("profile", "main")
                            .build(),
                    )
                    .build()?,
            ),
            _ => todo!(),
        };

        let h264parse = gstreamer::ElementFactory::make("h264parse")
            .name("h264_parser")
            .property("config-interval", 1)
            .build()?;

        let appsink = gstreamer_app::AppSink::builder()
            .name("appsink")
            .sync(false)
            .async_(false)
            .max_buffers(1)
            .drop(true)
            .build();

        pipeline.add_many([
            appsrc.upcast_ref(),
            &encoder,
            &capsfilter,
            &h264parse,
            appsink.upcast_ref(),
        ])?;

        appsrc.link(&encoder)?;
        encoder.link(&capsfilter)?;
        capsfilter.link(&h264parse)?;
        h264parse.link(&appsink)?;

        Ok(Self {
            rtp_pipeline,
            pipeline: PipelineWrapper(pipeline),
            appsrc,
            appsink,
            started: AtomicBool::new(false),
            stopped: AtomicBool::new(false),
        })
    }
}

impl GoliathGstPipeline for EncodingPipline {
    fn get_pipeline(&self) -> &gstreamer::Pipeline {
        &self.pipeline.0
    }

    fn start_pipeline(
        &self,
        input_caps: Option<&gstreamer::Caps>,
    ) -> Result<(), GoliathVideoError> {
        self.appsrc.set_caps(input_caps);

        self.appsink.set_callbacks(
            gstreamer_app::AppSinkCallbacks::builder()
                .new_sample({
                    let rtp_pipeline = Arc::clone(&self.rtp_pipeline);
                    move |appsink| {
                        let sample = appsink.pull_sample().map_err(|err| {
                            log::error!("Failed to pull sample from appsink: {}", err);
                            gstreamer::FlowError::Error
                        })?;

                        rtp_pipeline.push_sample(sample)
                    }
                })
                .build(),
        );

        let state_change = self.get_pipeline().set_state(gstreamer::State::Playing)?;
        if state_change != gstreamer::StateChangeSuccess::Success {
            log::warn!("State was not immediately set to playing, could async behaviour be on?");
        }

        self.started.store(true, Ordering::Relaxed);
        Ok(())
    }

    fn stop_pipeline(&self) -> Result<(), GoliathVideoError> {
        self.rtp_pipeline.stop_pipeline()?;
        if self.stopped.load(Ordering::Relaxed) {
            return Ok(());
        }

        let state_change = self.get_pipeline().set_state(gstreamer::State::Null)?;
        if state_change != gstreamer::StateChangeSuccess::Success {
            log::warn!(
                "Pipeline state change was not regular success, could async behaviour be on?"
            );
        }
        self.stopped.store(true, Ordering::Relaxed);

        Ok(())
    }
}

impl GoliathGstAppsrc for EncodingPipline {
    fn push_sample(&self, sample: gstreamer::Sample) -> Result<gst::FlowSuccess, gst::FlowError> {
        if !self.started.load(Ordering::Relaxed) {
            self.start_pipeline(sample.caps_owned().as_ref())
                .map_err(|err| {
                    log::error!("Failed to start encoding pipeline: {err}");
                    gst::FlowError::CustomError
                })?;
        }

        self.appsrc.push_sample(&sample)
    }
}
