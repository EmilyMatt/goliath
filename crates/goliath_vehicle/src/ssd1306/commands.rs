use tinyvec::{ArrayVec, array_vec};

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub(crate) enum AddressingMode {
    Horizontal = 0,
    Vertical = 1,
    Page = 2,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub(crate) enum VComHDeselectLevel {
    VCOM065 = 0x00,
    VCOM077 = 0x20,
    VCOM083 = 0x30,
}

#[derive(Debug)]
pub(crate) enum SSD1306Command {
    // Fundamental commands
    SetContrast(u8),
    EntireDisplay(bool),
    Inverse(bool),
    DisplayOn(bool),
    // Scrolling commands
    ContinuousHorizontalScroll {
        left: bool,
        start_page: u8,
        time_interval: u8,
        end_page: u8,
    },
    ContinuousVerticalAndHorizontalScroll {
        left: bool,
        start_page: u8,
        time_interval: u8,
        end_page: u8,
        vertical_offset: u8,
    },
    ScrollActivation(bool),
    SetVerticalScrollArea {
        num_fixed_rows: u8,
        num_scrolling_rows: u8,
    },
    // Addressing commands
    SetColumnBoundForAddressing {
        higher_bound: bool,
        column: u8,
    },
    SetAddressingMode(AddressingMode),
    SetColumnAddress {
        start: u8,
        end: u8,
    },
    SetPageAddress {
        start: u8,
        end: u8,
    },
    SetPageStartAddress(u8),
    // Hardware configuration commands
    SetDisplayStartLine(u8),
    SetSegmentRemap(bool),
    SetMultiplexRatio(u8),
    SetComOutputScanDirection(bool),
    SetDisplayOffset(u8),
    SetComPinsHardwareConfiguration {
        sequential: bool,
        remapped: bool,
    },
    // Timing and driving scheme setting commands
    SetDisplayClockDivRatioOscillatorFrequency {
        ratio: u8,
        frequency: u8,
    },
    SetPreChargePeriod {
        // first 4 bits
        phase_1: u8,
        // last 4 bits
        phase_2: u8,
    },
    SetVComHDeselectLevel(VComHDeselectLevel),
    Nop,
    ChargePump(bool),
}

impl SSD1306Command {
    pub(crate) fn get_bytes(&self) -> ArrayVec<[u8; 8]> {
        match self {
            SSD1306Command::SetContrast(value) => {
                array_vec!(0x81, *value)
            }
            SSD1306Command::EntireDisplay(on) => {
                array_vec!(0xA4 | *on as u8)
            }
            SSD1306Command::Inverse(on) => {
                array_vec!(0xA6 | *on as u8)
            }
            SSD1306Command::DisplayOn(on) => {
                array_vec!(0xAE | *on as u8)
            }
            SSD1306Command::ContinuousHorizontalScroll {
                left,
                start_page,
                time_interval,
                end_page,
            } => {
                array_vec!(
                    0x26 | *left as u8,
                    0x00,
                    *start_page,
                    *time_interval,
                    *end_page,
                    0x00,
                    0xFF
                )
            }
            SSD1306Command::ContinuousVerticalAndHorizontalScroll {
                left,
                start_page,
                time_interval,
                end_page,
                vertical_offset,
            } => {
                let scroll_direction = 0x28 | (0x01 << *left as u8);
                array_vec!(
                    scroll_direction,
                    0x00,
                    *start_page,
                    *time_interval,
                    *end_page,
                    *vertical_offset
                )
            }
            SSD1306Command::ScrollActivation(on) => {
                array_vec!(0x2E | *on as u8)
            }
            SSD1306Command::SetVerticalScrollArea {
                num_fixed_rows,
                num_scrolling_rows,
            } => {
                array_vec!(0xA3, *num_fixed_rows, *num_scrolling_rows)
            }
            SSD1306Command::SetColumnBoundForAddressing {
                higher_bound,
                column,
            } => {
                let higher_bound = (*higher_bound as u8) << 4;
                array_vec!(higher_bound | *column & 0x0F)
            }
            SSD1306Command::SetAddressingMode(addressing_mode) => {
                array_vec!(0x20, *addressing_mode as u8)
            }
            SSD1306Command::SetColumnAddress { start, end } => {
                array_vec!(0x21, *start, *end)
            }
            SSD1306Command::SetPageAddress { start, end } => {
                array_vec!(0x22, *start, *end)
            }
            SSD1306Command::SetPageStartAddress(start_address) => {
                array_vec!(0xB0 | *start_address & 0x0F)
            }
            SSD1306Command::SetDisplayStartLine(start_line) => {
                array_vec!(0x40 | *start_line & 0x3F)
            }
            SSD1306Command::SetSegmentRemap(remap) => {
                array_vec!(0x0A | *remap as u8)
            }
            SSD1306Command::SetMultiplexRatio(ratio) => {
                array_vec!(0xA8, *ratio & 0x3F)
            }
            SSD1306Command::SetComOutputScanDirection(remapped) => {
                array_vec!(0xC0 | (*remapped as u8) << 3)
            }
            SSD1306Command::SetDisplayOffset(offset) => {
                array_vec!(0xD3, *offset & 0x3F)
            }
            SSD1306Command::SetComPinsHardwareConfiguration {
                sequential,
                remapped,
            } => {
                array_vec!(
                    0x00,
                    0xDA,
                    0x02 | (*sequential as u8) << 4 | (*remapped as u8) << 5
                )
            }
            SSD1306Command::SetDisplayClockDivRatioOscillatorFrequency { ratio, frequency } => {
                array_vec!(0xD5, *ratio & 0x0F | ((*frequency & 0x0F) << 4))
            }
            SSD1306Command::SetPreChargePeriod { phase_1, phase_2 } => {
                array_vec!(0xD9, *phase_1 & 0x0F | ((*phase_2 & 0x0F) << 4))
            }
            SSD1306Command::SetVComHDeselectLevel(level) => {
                array_vec!(0xDB, *level as u8 & 0x70)
            }
            SSD1306Command::Nop => {
                array_vec!(0xE3)
            }
            SSD1306Command::ChargePump(enabled) => {
                array_vec!(0x8D, 0x10 | (*enabled as u8) << 2)
            }
        }
    }

    pub(crate) fn get_len(&self) -> usize {
        match self {
            SSD1306Command::Nop
            | SSD1306Command::DisplayOn(_)
            | SSD1306Command::SetDisplayStartLine(_)
            | SSD1306Command::SetSegmentRemap(_)
            | SSD1306Command::SetComOutputScanDirection(_)
            | SSD1306Command::EntireDisplay(_)
            | SSD1306Command::Inverse(_)
            | SSD1306Command::ScrollActivation(_)
            | SSD1306Command::SetColumnBoundForAddressing { .. }
            | SSD1306Command::SetPageStartAddress(_) => 1,
            SSD1306Command::SetContrast(_)
            | SSD1306Command::SetAddressingMode(_)
            | SSD1306Command::SetMultiplexRatio(_)
            | SSD1306Command::SetDisplayOffset(_)
            | SSD1306Command::SetDisplayClockDivRatioOscillatorFrequency { .. }
            | SSD1306Command::SetPreChargePeriod { .. }
            | SSD1306Command::SetVComHDeselectLevel(_)
            | SSD1306Command::ChargePump(_) => 2,
            SSD1306Command::SetVerticalScrollArea { .. }
            | SSD1306Command::SetComPinsHardwareConfiguration { .. }
            | SSD1306Command::SetPageAddress { .. }
            | SSD1306Command::SetColumnAddress { .. } => 3,
            SSD1306Command::ContinuousVerticalAndHorizontalScroll { .. } => 6,
            SSD1306Command::ContinuousHorizontalScroll { .. } => 7,
        }
    }
}
