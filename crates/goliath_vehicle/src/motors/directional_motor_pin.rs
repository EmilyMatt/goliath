use crate::GoliathVehicleResult;
use jetgpio::Pwm;
use jetgpio::gpio::pins::OutputPin;

pub(crate) struct DirectionalMotorPin {
    power: Pwm,
    forward: OutputPin,
    backward: OutputPin,
}

impl DirectionalMotorPin {
    pub(crate) fn try_new(
        power: Pwm,
        forward: OutputPin,
        backward: OutputPin,
    ) -> GoliathVehicleResult<Self> {
        Ok(Self {
            power,
            forward,
            backward,
        })
    }

    pub(crate) fn set_power_and_direction(
        &mut self,
        power: f64,
        forward: bool,
    ) -> GoliathVehicleResult<()> {
        assert!((0.0..=1.0).contains(&power));

        if forward {
            self.forward.set_high()?;
            self.backward.set_low()?;
        } else {
            self.forward.set_low()?;
            self.backward.set_high()?;
        }

        self.power.set_duty_cycle((power * 100.0).floor() as u32)?;

        Ok(())
    }
}
