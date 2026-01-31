use crate::GoliathSerdeError;
use bytes::Bytes;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum GoliathReport {}

impl GoliathReport {
    pub fn read_from_bytes(msg_bytes: &[u8]) -> Result<Self, GoliathSerdeError> {
        let cmd = bitcode::deserialize(msg_bytes)?;
        Ok(cmd)
    }

    pub fn into_bytes(self) -> Result<Bytes, GoliathSerdeError> {
        let data = bitcode::serialize(&self)?;
        Ok(Bytes::from_owner(data))
    }
}
