use crate::GoliathVehicleResult;
use crate::ssd1306::commands::{AddressingMode, VComHDeselectLevel};
use jetgpio::I2c;
use jetgpio::i2c::bus::I2cBus;
use tinyvec::ArrayVec;

use crate::error::GoliathVehicleError;
use crate::ssd1306::screen_space::ScreenSpace;
pub(crate) use commands::SSD1306Command;

#[allow(unused)]
mod commands;
mod screen_space;

pub(crate) struct SSD1306 {
    interface: I2c,
    command_buffer: ArrayVec<[u8; 8]>, // Maximum buffer for a single command
    data_buffer: ArrayVec<[u8; 256]>, // Must be enough to contain 129 bytes(4 pages + control byte), but most be in exponent of 2
    screen_space: ScreenSpace,
}

impl SSD1306 {
    pub(crate) fn new(interface: I2c, width: u32, height: u32) -> Self {
        SSD1306 {
            interface,
            command_buffer: ArrayVec::from_array_len([0x00; 8], 1),
            data_buffer: ArrayVec::from_array_len([0x40; 256], 129), // 128 bytes of data + 1 control byte
            screen_space: ScreenSpace::new(width, height),
        }
    }

    pub(crate) fn builder(interface: I2c, width: u32, height: u32) -> SSD1306Builder {
        SSD1306Builder {
            interface,
            oscillator_frequency: 0x8,
            clock_divide_ratio: 0x0,
            width,
            height,
            contrast: 0x0F,
        }
    }

    pub(crate) fn send_command(&mut self, command: SSD1306Command) -> GoliathVehicleResult<()> {
        self.command_buffer.set_len(command.get_len() + 1);
        self.command_buffer[1..].copy_from_slice(&command.get_bytes());

        self.interface
            .write(self.command_buffer[0], &self.command_buffer[1..])
            .map_err(Into::into)
    }

    #[cfg_attr(feature = "trace", tracing::instrument(level = "info", skip_all))]
    pub(crate) fn send_data_to_screen(&mut self) -> GoliathVehicleResult<()> {
        let pages_to_update = self.screen_space.take_pages_to_update();
        if !pages_to_update.contains(&true) {
            // No pages to update, nothing to send
            return Ok(());
        }

        let screen_width = self.screen_space.width() as usize;
        pages_to_update
            .into_iter()
            .enumerate()
            .filter_map(|(page_idx, should_write)| should_write.then_some(page_idx))
            .try_for_each(|page_idx| {
                self.send_command(SSD1306Command::SetPageAddress {
                    start: page_idx as u8,
                    end: page_idx as u8,
                })?;

                self.send_command(SSD1306Command::SetColumnAddress {
                    start: 0,
                    end: screen_width as u8 - 1,
                })?;

                self.data_buffer[1..].copy_from_slice(
                    &self.screen_space.buffer()
                        [page_idx * screen_width..(page_idx + 1) * screen_width],
                );

                self.interface
                    .write(self.data_buffer[0], &self.data_buffer[1..])
                    .map_err(Into::into)
            })
    }

    pub(crate) fn clear_screen(&mut self) -> GoliathVehicleResult<()> {
        self.screen_space.clear();

        self.send_data_to_screen()
    }

    pub(crate) fn update_screen(&mut self, pos: usize, data: &[u8]) -> GoliathVehicleResult<()> {
        if pos + data.len() > self.screen_space.buffer_size() {
            return Err(GoliathVehicleError::WriteError(
                "Position out of bounds for screen space".to_string(),
            ));
        }
        self.screen_space.update(pos, data);

        self.send_data_to_screen()
    }

    pub(crate) fn width(&self) -> u32 {
        self.screen_space.width()
    }

    pub(crate) fn height(&self) -> u32 {
        self.screen_space.height()
    }
}

// Try shutting down display on drop
impl Drop for SSD1306 {
    fn drop(&mut self) {
        self.send_command(SSD1306Command::DisplayOn(false)).ok();
    }
}

pub(crate) struct SSD1306Builder {
    interface: I2c,
    oscillator_frequency: u8,
    clock_divide_ratio: u8,
    width: u32,
    height: u32,
    contrast: u8,
}

impl SSD1306Builder {
    pub fn with_oscillator_frequency(self, frequency: u8) -> Self {
        Self {
            oscillator_frequency: frequency,
            ..self
        }
    }

    pub fn with_clock_divide_ratio(self, ratio: u8) -> Self {
        Self {
            clock_divide_ratio: ratio,
            ..self
        }
    }

    pub fn with_contrast(self, contrast: u8) -> Self {
        Self { contrast, ..self }
    }

    #[cfg_attr(feature = "trace", tracing::instrument(level = "info", skip_all))]
    pub fn build(self) -> GoliathVehicleResult<SSD1306> {
        let commands = [
            SSD1306Command::DisplayOn(false),
            SSD1306Command::SetDisplayClockDivRatioOscillatorFrequency {
                frequency: self.oscillator_frequency,
                ratio: self.clock_divide_ratio,
            },
            SSD1306Command::SetMultiplexRatio(self.height as u8 - 1),
            SSD1306Command::SetDisplayOffset(0),
            SSD1306Command::SetDisplayStartLine(0),
            SSD1306Command::ChargePump(true),
            SSD1306Command::SetAddressingMode(AddressingMode::Page),
            SSD1306Command::SetSegmentRemap(false),
            SSD1306Command::SetComOutputScanDirection(false),
            SSD1306Command::SetComPinsHardwareConfiguration {
                sequential: false,
                remapped: false,
            },
            SSD1306Command::SetSegmentRemap(true), // Otherwise everything is upside down??
            SSD1306Command::SetComOutputScanDirection(false),
            SSD1306Command::SetPreChargePeriod {
                phase_1: 0x1,
                phase_2: 0x2, // Higher than 2 has no effect on most devices
            },
            SSD1306Command::SetContrast(self.contrast),
            SSD1306Command::SetVComHDeselectLevel(VComHDeselectLevel::VCOM077),
            SSD1306Command::EntireDisplay(false),
            SSD1306Command::Inverse(false),
            SSD1306Command::ScrollActivation(false),
            SSD1306Command::DisplayOn(true),
        ];

        let mut new_ssd = SSD1306::new(self.interface, self.width, self.height);
        commands
            .into_iter()
            .try_for_each(|command| new_ssd.send_command(command))?;

        Ok(new_ssd)
    }
}

pub(crate) fn create_ssd_connection() -> GoliathVehicleResult<SSD1306> {
    let mut i2c = I2c::init(I2cBus::I2c1, 0)?;
    i2c.set_slave_address(0x3C);

    let mut ssd = SSD1306::builder(i2c, 128, 32)
        .with_oscillator_frequency(0x8)
        .with_clock_divide_ratio(0x0)
        .with_contrast(0x0F)
        .build()?;

    ssd.clear_screen()?;

    Ok(ssd)
}
