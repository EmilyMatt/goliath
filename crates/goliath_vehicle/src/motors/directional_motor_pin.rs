use crate::GoliathVehicleResult;
use rppal::gpio::{Gpio, OutputPin};
use rppal::pwm::{Channel, Pwm};

const PWM_MAX_FREQUENCY: f64 = 8192.0; // 8kHz is the maximum frequency for the Raspberry Pi's hardware PWM

pub(crate) struct DirectionalMotorPin {
    power: Pwm,
    forward: OutputPin,
    backward: OutputPin,
}

impl DirectionalMotorPin {
    pub(crate) fn try_new(
        pwm_channel: Channel,
        forward_pin: u8,
        backward_pin: u8,
    ) -> GoliathVehicleResult<Self> {
        let power = Pwm::with_frequency(
            pwm_channel,
            PWM_MAX_FREQUENCY,
            0.0,
            rppal::pwm::Polarity::Normal,
            true,
        )?;

        let gpio = Gpio::new()?;
        let mut forward = gpio.get(forward_pin)?.into_output();
        forward.set_low();
        let mut backward = gpio.get(backward_pin)?.into_output();
        backward.set_low();

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
            self.forward.set_high();
            self.backward.set_low();
        } else {
            self.forward.set_low();
            self.backward.set_high();
        }

        self.power.set_duty_cycle(power)?;

        Ok(())
    }
}
