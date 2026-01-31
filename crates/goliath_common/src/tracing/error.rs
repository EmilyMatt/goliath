#[derive(thiserror::Error, Debug)]
pub enum GoliathTracingError {
    #[error("Error initializing logger: {0}")]
    SetLoggerError(#[from] log::SetLoggerError),

    #[cfg(feature = "trace")]
    #[error("Tracing Initialization error: {0}")]
    TracingInitError(#[from] tracing_subscriber::util::TryInitError),
}
