#[derive(thiserror::Error, Debug)]
pub enum GoliathVideoError {
    #[error("Video pipeline error: {0}")]
    GeneralError(String),

    #[error("GStreamer state change error: {0}")]
    StateChangeError(#[from] gstreamer::StateChangeError),

    #[error("GStreamer flow error: {0}")]
    FlowError(#[from] gstreamer::FlowError),
}
