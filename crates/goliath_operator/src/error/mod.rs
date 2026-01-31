use goliath_common::GoliathTracingError;
use sdl2::IntegerOrSdlError;
use sdl2::video::WindowBuildError;

pub(crate) type GoliathOperatorResult<T> = Result<T, GoliathOperatorError>;

#[allow(dead_code)]
#[derive(thiserror::Error, Debug)]
pub(crate) enum GoliathOperatorError {
    #[error("General error: {0}")]
    GeneralError(String),

    #[error("Not all bytes were written: {0}")]
    WriteError(String),

    #[error("SDL2 context error: {0}")]
    SdlError(String),

    #[error("Image error: {0}")]
    ImageError(#[from] image::ImageError),

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

    #[error("Error while initializing logging/tracing: {0}")]
    TracingInitError(#[from] GoliathTracingError),

    #[error("Error while creating application window: {0}")]
    WindowCreationError(#[from] WindowBuildError),

    #[error("Overflow or SDL Error: {0}")]
    IntegerOrSdlError(#[from] IntegerOrSdlError),
}
