#[derive(thiserror::Error, Debug)]
pub enum GoliathSerdeError {
    #[error("Error deserializing JSON message: {0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("Error deserializing wincode message: {0}")]
    BitcodeDecodeError(#[from] bitcode::Error),
}
