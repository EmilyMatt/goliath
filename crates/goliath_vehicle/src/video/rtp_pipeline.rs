use crate::error::GoliathVehicleResult;
use goliath_common::{GoliathGstAppsrc, GoliathGstPipeline, GoliathVideoError, PipelineWrapper};
use gstreamer::ClockTime;
use gstreamer::prelude::{Cast, ElementExt, ElementExtManual, GstBinExtManual};
use gstreamer_app::gst;
use std::sync::atomic::{AtomicBool, Ordering};

pub(crate) struct RTPPipeline {
    pipeline: PipelineWrapper,
    appsrc: gstreamer_app::AppSrc,
    started: AtomicBool,
    stopped: AtomicBool,
}

impl RTPPipeline {
    pub(crate) fn try_new(output_uris: Vec<(String, u16)>) -> GoliathVehicleResult<Self> {
        let pipeline = gstreamer::Pipeline::builder()
            .name("RTPPipeline")
            .async_handling(false)
            .latency(ClockTime::from_mseconds(0))
            .build();

        let appsrc = gstreamer_app::AppSrc::builder()
            .name("appsrc")
            .do_timestamp(true)
            .format(gstreamer::Format::Time)
            .build();

        let rtp264pay = gstreamer::ElementFactory::make("rtph264pay")
            .name("rtp_payloader")
            .property("config-interval", 1)
            .build()?;

        let clients_list = output_uris
            .into_iter()
            .map(|(ip, port)| format!("{ip}:{port}"))
            .collect::<Vec<_>>()
            .join(",");

        log::info!("Sending to clients: {clients_list:?}");
        let udpsink = gstreamer::ElementFactory::make("udpsink")
            .name("udp_sink")
            .property("clients", clients_list)
            .property("sync", false)
            .property("async", false)
            .build()?;

        pipeline.add_many([appsrc.upcast_ref(), &rtp264pay, &udpsink])?;

        appsrc.link(&rtp264pay)?;
        rtp264pay.link(&udpsink)?;

        Ok(Self {
            pipeline: PipelineWrapper::wrap(pipeline),
            appsrc,
            started: AtomicBool::new(false),
            stopped: AtomicBool::new(false),
        })
    }
}

impl GoliathGstPipeline for RTPPipeline {
    fn get_pipeline(&self) -> &gstreamer::Pipeline {
        self.pipeline.as_ref()
    }

    fn start_pipeline(
        &self,
        input_caps: Option<&gstreamer::Caps>,
    ) -> Result<(), GoliathVideoError> {
        self.appsrc.set_caps(input_caps);

        let state_change = self.get_pipeline().set_state(gstreamer::State::Playing)?;
        if state_change != gstreamer::StateChangeSuccess::Success {
            self.get_pipeline().set_state(gstreamer::State::Null)?;
            return Err(GoliathVideoError::GeneralError(
                "State was not set to Playing".into(),
            ));
        }

        self.started.store(true, Ordering::Relaxed);
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

impl GoliathGstAppsrc for RTPPipeline {
    fn push_sample(&self, sample: gstreamer::Sample) -> Result<gst::FlowSuccess, gst::FlowError> {
        if !self.started.load(Ordering::Relaxed) {
            self.start_pipeline(sample.caps_owned().as_ref())
                .map_err(|err| {
                    log::error!("Could not start RTP pipeline: {err}");
                    gst::FlowError::CustomError
                })?;
        }

        self.appsrc.push_sample(&sample)
    }
}
