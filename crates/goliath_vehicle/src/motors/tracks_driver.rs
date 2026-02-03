use crate::GoliathVehicleResult;
use crate::motors::directional_motor_pin::DirectionalMotorPin;
use jetgpio::Gpio;
use jetgpio::gpio::valid_pins;
use jetgpio::pwm::Pwm;
use std::sync::Arc;

pub(crate) struct TracksDriver {
    left_track: DirectionalMotorPin,
    right_track: DirectionalMotorPin,
    thrust: f32,
    steer: f32,
}

impl TracksDriver {
    pub(crate) fn try_new(gpio: &Arc<Gpio>) -> GoliathVehicleResult<Self> {
        let mut left_track = DirectionalMotorPin::try_new(
            Pwm::new(valid_pins::Pin32)?,
            gpio.get_output(valid_pins::Pin3)?,
            gpio.get_output(valid_pins::Pin4)?,
        )?;
        left_track.set_power_and_direction(0.0, true)?; // Ensure the motor is stopped on init

        let mut right_track = DirectionalMotorPin::try_new(
            Pwm::new(valid_pins::Pin32)?,
            gpio.get_output(valid_pins::Pin3)?,
            gpio.get_output(valid_pins::Pin4)?,
        )?;
        right_track.set_power_and_direction(0.0, true)?; // Ensure the motor is stopped on init

        Ok(Self {
            left_track,
            right_track,
            thrust: 0.0,
            steer: 0.0,
        })
    }

    pub(crate) fn set_thrust(&mut self, thrust: f32) {
        // Thrust has to be between -1.0 and 1.0
        assert!((-1.0..=1.0).contains(&thrust));
        self.thrust = thrust;
    }

    pub(crate) fn set_steer(&mut self, steer: f32) {
        // Ratio between the left and right tracks has to be between -1.0 and 1.0
        assert!((-1.0..=1.0).contains(&steer));
        self.steer = steer;
    }

    pub(crate) fn update_tracks(&mut self) -> GoliathVehicleResult<()> {
        let max_steer = if self.thrust.abs() <= 0.0 {
            1.0 // Avoid division by zero
        } else {
            (1.0 - self.thrust.abs()) / self.thrust.abs()
        };

        let constrained_steer = self.steer.clamp(-max_steer, max_steer);

        let left_power = self.thrust * (1.0 - constrained_steer);
        let right_power = self.thrust * (1.0 + constrained_steer);

        let left_forward = left_power >= 0.0;
        let right_forward = right_power >= 0.0;

        self.left_track
            .set_power_and_direction(left_power.abs() as f64, left_forward)?;
        log::info!(
            "Left track power set to {}, moving forward: {left_forward}",
            left_power.abs()
        );
        self.right_track
            .set_power_and_direction(right_power.abs() as f64, right_forward)?;
        log::info!(
            "Right track power set to {}, moving forward: {right_forward}",
            right_power.abs()
        );

        Ok(())
    }
}

impl Drop for TracksDriver {
    fn drop(&mut self) {
        if let Err(e) = self.left_track.set_power_and_direction(0.0, true) {
            log::error!("Failed to stop left track: {}", e);
        }
        if let Err(e) = self.right_track.set_power_and_direction(0.0, true) {
            log::error!("Failed to stop right track: {}", e);
        }
    }
}
