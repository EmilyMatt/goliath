use crate::GoliathVehicleResult;
use crate::error::GoliathVehicleError;
use crate::motors::tracks_driver::TracksDriver;
use crate::motors::turret_driver::TurretDriver;
use goliath_common::MotorCommand;
use jetgpio::Gpio;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::TryRecvError;

mod directional_motor_pin;
mod tracks_driver;
mod turret_driver;

pub(crate) struct MotorsContoller {
    tracks_driver: TracksDriver,
    _turret_driver: TurretDriver,
}

impl MotorsContoller {
    pub(crate) fn try_new(gpio: Arc<Gpio>) -> GoliathVehicleResult<Self> {
        Ok(Self {
            tracks_driver: TracksDriver::try_new(&gpio)?,
            _turret_driver: TurretDriver::new(&gpio)?,
        })
    }

    pub(crate) fn run_thread(
        &mut self,
        mut cmd_channel: mpsc::Receiver<MotorCommand>,
    ) -> GoliathVehicleResult<()> {
        let mut modified_tracks = false;
        let mut msg_count = 0;
        loop {
            if msg_count > 5 {
                if modified_tracks {
                    self.tracks_driver.update_tracks()?;
                    modified_tracks = false;
                }
                msg_count = 0;
            }
            match cmd_channel.try_recv() {
                Ok(cmd) => {
                    msg_count += 1;
                    match cmd {
                        MotorCommand::Thrust(thrust) => {
                            self.tracks_driver.set_thrust(thrust);
                            modified_tracks = true;
                        }
                        MotorCommand::Steer(steer) => {
                            self.tracks_driver.set_steer(steer);
                            modified_tracks = true;
                        }
                        MotorCommand::TurretAngle(_) => {}
                        MotorCommand::End => {
                            log::info!("Got END command, stopping motors");
                            self.tracks_driver.set_thrust(0.0);
                            self.tracks_driver.set_steer(0.0);
                            self.tracks_driver.update_tracks()?;
                            break;
                        }
                    }
                }
                Err(TryRecvError::Empty) => {
                    if modified_tracks {
                        self.tracks_driver.update_tracks()?;
                        modified_tracks = false;
                    }
                    msg_count = 0;
                }
                Err(e @ TryRecvError::Disconnected) => {
                    log::info!("Motor command channel disconnected, stopping motors.");
                    self.tracks_driver.set_thrust(0.0);
                    self.tracks_driver.set_steer(0.0);
                    self.tracks_driver.update_tracks()?;
                    return Err(GoliathVehicleError::from(e));
                }
            }
        }

        Ok(())
    }
}
