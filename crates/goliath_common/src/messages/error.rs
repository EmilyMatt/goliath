#[derive(thiserror::Error, Debug)]
pub enum GoliathSerdeError {
    #[error("Error deserializing JSON message: {0}")]
    SerdeJsonError(#[from] serde_json::Error),

    #[error("Error deserializing wincode message: {0}")]
    BitcodeError(#[from] bitcode::Error),
}
