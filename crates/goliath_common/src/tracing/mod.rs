mod error;

pub use error::GoliathTracingError;

pub fn common_init_for_trace() -> Result<(), GoliathTracingError> {
    #[cfg(feature = "trace")]
    {
        use tracing_subscriber::util::SubscriberInitExt;

        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::builder()
                    .with_default_directive(tracing::level_filters::LevelFilter::INFO.into())
                    .from_env_lossy(),
            )
            .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
            .finish()
            .try_init()?;
    }
    #[cfg(not(feature = "trace"))]
    {
        env_logger::try_init_from_env(
            env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "INFO"),
        )?;
    }

    Ok(())
}
