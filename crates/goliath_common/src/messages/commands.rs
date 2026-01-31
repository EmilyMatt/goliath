use crate::messages::error::GoliathSerdeError;

#[derive(serde::Serialize, serde::Deserialize)]
pub enum MotorCommand {
    Thrust(f32),
    Steer(f32),
    TurretAngle(f32),

    // Only manual construction
    #[serde(skip)]
    End,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub enum GoliathCommand {
    Motor(MotorCommand),
}

impl GoliathCommand {
    pub fn read_from_bytes(msg_bytes: &[u8]) -> Result<GoliathCommand, GoliathSerdeError> {
        let cmd = bitcode::deserialize(msg_bytes)?;
        Ok(cmd)
    }
}
