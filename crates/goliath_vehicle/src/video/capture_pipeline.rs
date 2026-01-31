use crate::error::GoliathVehicleResult;
use crate::video::PipelineWrapper;
use crate::video::encoding_pipeline::EncodingPipline;
use goliath_common::{GoliathGstAppsrc, GoliathGstPipeline, GoliathVideoError};
use gstreamer::prelude::{Cast, ElementExt, ElementExtManual, GstBinExtManual};
use gstreamer::{ClockTime, Fraction};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

// Note that the camera provides 2 views, so the resolution is *doubled* in width
#[derive(Copy, Clone)]
#[allow(dead_code)]
pub(crate) enum ZedCamCaps {
    UHD2K15, // 2K resolution at 15 FPS

    FHD1080P30, // Full HD(1080p) at 30 FPS
    FHD1080P15, // Full HD(1080p) at 15 FPS

    HD720P60, // HD(720p) at 60 FPS
    HD720P30, // HD(720p) at 30 FPS
    HD720P15, // HD(720p) at 15 FPS

    NOHD100, // No HD(672x376) at 100 FPS
    NOHD60,  // No HD(672x376) at 60 FPS
    NOHD30,  // No HD(672x376) at 30 FPS
    NOHD15,  // No HD(672x376) at 15 FPS
}

impl ZedCamCaps {
    pub(crate) fn get_caps(&self) -> gstreamer::Caps {
        let mut builder = gstreamer::Caps::builder("video/x-raw")
            .field("format", "NV12")
            .field("colorimetry", "bt709");

        builder = match self {
            Self::UHD2K15 => builder.field("width", 4416).field("height", 1242),
            Self::FHD1080P30 | ZedCamCaps::FHD1080P15 => {
                builder.field("width", 3840).field("height", 1080)
            }
            Self::HD720P60 | ZedCamCaps::HD720P30 | ZedCamCaps::HD720P15 => {
                builder.field("width", 2560).field("height", 720)
            }
            Self::NOHD100 | ZedCamCaps::NOHD60 | ZedCamCaps::NOHD30 | ZedCamCaps::NOHD15 => {
                builder.field("width", 1344).field("height", 376)
            }
        };

        let fps = match self {
            Self::NOHD100 => 100,
            Self::FHD1080P30 | Self::HD720P30 | Self::NOHD30 => 30,
            Self::HD720P60 | Self::NOHD60 => 60,
            Self::UHD2K15 | Self::FHD1080P15 | Self::HD720P15 | Self::NOHD15 => 15,
        };

        builder.field("framerate", Fraction::new(fps, 1)).build()
    }
}

pub(crate) struct CapturePipeline {
    encoding_pipline: Arc<dyn GoliathGstAppsrc>,
    pipeline: PipelineWrapper,
    appsink: gstreamer_app::AppSink,
    stopped: AtomicBool,
}

impl CapturePipeline {
    pub(crate) fn try_new(
        capture_caps: ZedCamCaps,
        encoding_pipline: Arc<EncodingPipline>,
    ) -> GoliathVehicleResult<Self> {
        let pipeline = gstreamer::Pipeline::builder()
            .name("CapturePipeline")
            .async_handling(false)
            .latency(ClockTime::from_mseconds(0))
            .build();

        let src = gstreamer::ElementFactory::make("v4l2src")
            .name("camera_source")
            .property("device", "/dev/video0")
            .property("do-timestamp", true)
            .build()?;

        let videoconvert = gstreamer::ElementFactory::make("videoconvert")
            .name("video_convert")
            .build()?;

        let caps = capture_caps.get_caps();

        let capsfilter = gstreamer::ElementFactory::make("capsfilter")
            .name("caps_filter")
            .property("caps", caps.clone())
            .build()?;

        let appsink = gstreamer_app::AppSink::builder()
            .name("appsink")
            .sync(false)
            .async_(false)
            .max_buffers(1)
            .drop(true)
            .caps(&caps)
            .build();

        pipeline.add_many([&src, &videoconvert, &capsfilter, appsink.upcast_ref()])?;

        src.link(&videoconvert)?;
        videoconvert.link(&capsfilter)?;
        capsfilter.link(&appsink)?;

        Ok(Self {
            encoding_pipline,
            pipeline: PipelineWrapper(pipeline),
            appsink,
            stopped: AtomicBool::new(false),
        })
    }
}

impl GoliathGstPipeline for CapturePipeline {
    fn get_pipeline(&self) -> &gstreamer::Pipeline {
        &self.pipeline.0
    }

    fn start_pipeline(&self, _: Option<&gstreamer::Caps>) -> Result<(), GoliathVideoError> {
        if self.stopped.load(Ordering::Relaxed) {
            return Err(GoliathVideoError::GeneralError(
                "Pipeline was already stopped, it no longer exists".to_string(),
            ));
        }

        self.appsink.set_callbacks(
            gstreamer_app::AppSinkCallbacks::builder()
                .new_sample({
                    let encoding_pipeline = Arc::clone(&self.encoding_pipline);
                    move |appsink| {
                        let sample = appsink.pull_sample().map_err(|err| {
                            log::error!("Failed to pull sample from appsink: {}", err);
                            gstreamer::FlowError::Error
                        })?;

                        encoding_pipeline.push_sample(sample)
                    }
                })
                .build(),
        );

        let state_change = self.get_pipeline().set_state(gstreamer::State::Playing)?;
        if state_change != gstreamer::StateChangeSuccess::Success {
            log::warn!("State was not immediately set to playing, could async behaviour be on?");
        }

        Ok(())
    }

    fn stop_pipeline(&self) -> Result<(), GoliathVideoError> {
        self.encoding_pipline.stop_pipeline()?;
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
