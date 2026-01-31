use goliath_common::{GoliathTracingError, GoliathVideoError};
use gstreamer::glib;

pub(crate) type GoliathVehicleResult<T> = Result<T, GoliathVehicleError>;

#[derive(thiserror::Error, Debug)]
pub(crate) enum GoliathVehicleError {
    #[error("General error: {0}")]
    GeneralError(String),

    #[error("I2C error: {0}")]
    I2C(#[from] rppal::i2c::Error),

    #[error("PWM error: {0}")]
    Pwm(#[from] rppal::pwm::Error),

    #[error("GPIO error: {0}")]
    Gpio(#[from] rppal::gpio::Error),

    #[error("Not all bytes were written: {0}")]
    WriteError(String),

    #[error("Image error: {0}")]
    ImageError(#[from] image::ImageError),

    #[error("GStreamer error: {0}")]
    GlibError(#[from] glib::Error),

    #[error("GStreamer initialization error: {0}")]
    GlibBoolError(#[from] glib::BoolError),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Async channel receive error: {0}")]
    TokioTryRecvError(#[from] tokio::sync::mpsc::error::TryRecvError),

    #[error("WebSocket error: {0}")]
    WSError(#[from] Box<tokio_tungstenite::tungstenite::Error>),

    #[error("Tokio Join error: {0}")]
    TokioJoinError(#[from] tokio::task::JoinError),

    #[error("Tokio Send error: {0}")]
    TokioSendError(String),

    #[error("Video pipeline error: {0}")]
    VideoError(#[from] GoliathVideoError),

    #[error("Error while initializing logging/tracing: {0}")]
    TracingInitError(#[from] GoliathTracingError),
}
