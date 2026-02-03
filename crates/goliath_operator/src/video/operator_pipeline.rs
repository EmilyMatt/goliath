use crate::error::GoliathOperatorResult;
use goliath_common::{GoliathGstPipeline, GoliathVideoError, PipelineWrapper};
use gstreamer::ClockTime;
use gstreamer::prelude::{ElementExt, ElementExtManual, GObjectExtManualGst, GstBinExtManual};
use std::sync::atomic::{AtomicBool, Ordering};

pub(crate) struct OperatorPipeline {
    pipeline: PipelineWrapper,
    stopped: AtomicBool,
}

impl OperatorPipeline {
    pub(crate) fn try_new(rtp_caps: gstreamer::Caps) -> GoliathOperatorResult<Self> {
        let pipeline = gstreamer::Pipeline::builder()
            .name("CapturePipeline")
            .async_handling(false)
            .latency(ClockTime::from_mseconds(0))
            .build();

        let src = gstreamer::ElementFactory::make("udpsrc")
            .name("udp_source")
            .property("port", 9000)
            .build()?;

        let capsfilter = gstreamer::ElementFactory::make("capsfilter")
            .name("caps_filter")
            .property("caps", rtp_caps)
            .build()?;

        let depayloader = gstreamer::ElementFactory::make("rtph264depay")
            .name("depayloader")
            .build()?;

        let h264parse = gstreamer::ElementFactory::make("h264parse")
            .name("parser")
            .build()?;

        let decoder = gstreamer::ElementFactory::make("avdec_h264")
            .name("decoder")
            .build()?;

        let queue = gstreamer::ElementFactory::make("queue")
            .name("queue")
            .build()?;

        queue.set_property_from_str("leaky", "downstream");

        let videoconvert = gstreamer::ElementFactory::make("videoconvert")
            .name("format_converter")
            .build()?;

        let ximagesink = gstreamer::ElementFactory::make("ximagesink")
            .name("video_window")
            .property("sync", false)
            .property("async", false)
            .build()?;

        pipeline.add_many([
            &src,
            &capsfilter,
            &depayloader,
            &h264parse,
            &decoder,
            &queue,
            &videoconvert,
            &ximagesink,
        ])?;

        src.link(&capsfilter)?;
        capsfilter.link(&depayloader)?;
        depayloader.link(&h264parse)?;
        h264parse.link(&decoder)?;
        decoder.link(&queue)?;
        queue.link(&videoconvert)?;
        videoconvert.link(&ximagesink)?;

        Ok(Self {
            pipeline: PipelineWrapper::wrap(pipeline),
            stopped: AtomicBool::new(false),
        })
    }
}

impl GoliathGstPipeline for OperatorPipeline {
    fn get_pipeline(&self) -> &gstreamer::Pipeline {
        self.pipeline.as_ref()
    }

    fn start_pipeline(
        &self,
        _input_caps: Option<&gstreamer::caps::Caps>,
    ) -> Result<(), GoliathVideoError> {
        if self.stopped.load(Ordering::Relaxed) {
            return Err(GoliathVideoError::GeneralError(
                "Pipeline was already stopped, it no longer exists".to_string(),
            ));
        }

        let state_change = self.get_pipeline().set_state(gstreamer::State::Playing)?;
        if state_change != gstreamer::StateChangeSuccess::Success {
            log::warn!("State was not immediately set to playing, could async behaviour be on?");
        }

        log::info!("Pipeline started");
        Ok(())
    }

    fn stop_pipeline(&self) -> Result<(), GoliathVideoError> {
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
