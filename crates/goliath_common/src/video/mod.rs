use gstreamer::prelude::ElementExt;
use lazy_static::lazy_static;

mod error;
mod pipeline;

pub use error::GoliathVideoError;
pub use pipeline::{GoliathGstAppsrc, GoliathGstPipeline};

lazy_static! {
    static ref main_loop: Option<gstreamer::glib::MainLoop> =
        Some(gstreamer::glib::MainLoop::new(None, false));
}

pub fn start_main_loop() {
    main_loop.iter().for_each(|l| l.run());
}

pub fn stop_main_loop() {
    main_loop.iter().for_each(|l| l.quit());
}

pub fn initiate_gstreamer() -> Result<(), GoliathVideoError> {
    gstreamer::init().map_err(Into::into)
}

pub struct PipelineWrapper(gstreamer::Pipeline);

impl PipelineWrapper {
    pub fn wrap(pipeline: gstreamer::Pipeline) -> Self {
        Self(pipeline)
    }
}

impl AsRef<gstreamer::Pipeline> for PipelineWrapper {
    fn as_ref(&self) -> &gstreamer::Pipeline {
        &self.0
    }
}

impl Drop for PipelineWrapper {
    fn drop(&mut self) {
        self.0.set_state(gstreamer::State::Null).ok();
    }
}
