pub(crate) mod capture_pipeline;
pub(crate) mod encoding_pipeline;
pub(crate) mod rtp_pipeline;

use crate::GoliathVehicleResult;
use gstreamer::prelude::ElementExt;

pub(crate) fn initiate_gstreamer() -> GoliathVehicleResult<()> {
    gstreamer::init().map_err(Into::into)
}

pub(crate) struct PipelineWrapper(pub(crate) gstreamer::Pipeline);

impl Drop for PipelineWrapper {
    fn drop(&mut self) {
        self.0.set_state(gstreamer::State::Null).ok();
    }
}
