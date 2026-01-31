use crate::video::error::GoliathVideoError;
use gstreamer::Sample;
use gstreamer_app::gst;

pub trait GoliathGstPipeline: Sync + Send {
    fn get_pipeline(&self) -> &gstreamer::Pipeline;
    fn start_pipeline(&self, input_caps: Option<&gstreamer::Caps>)
    -> Result<(), GoliathVideoError>;
    fn stop_pipeline(&self) -> Result<(), GoliathVideoError>;
}

pub trait GoliathGstAppsrc: GoliathGstPipeline {
    fn push_sample(&self, sample: Sample) -> Result<gst::FlowSuccess, gst::FlowError>;
}
