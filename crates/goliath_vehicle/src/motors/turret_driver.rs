use crate::error::GoliathVehicleResult;
use jetgpio::Gpio;
use std::sync::Arc;

pub(crate) struct TurretDriver {}

impl TurretDriver {
    pub(crate) fn new(_gpio: &Arc<Gpio>) -> GoliathVehicleResult<Self> {
        Ok(Self {})
    }
}
