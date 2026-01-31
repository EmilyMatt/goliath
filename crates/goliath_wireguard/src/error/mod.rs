use goliath_common::GoliathTracingError;

pub(crate) type GoliathWireguardResult<T> = Result<T, GoliathWireguardError>;

#[allow(dead_code)]
#[derive(thiserror::Error, Debug)]
pub(crate) enum GoliathWireguardError {
    #[error("General error: {0}")]
    GeneralError(String),

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
}
