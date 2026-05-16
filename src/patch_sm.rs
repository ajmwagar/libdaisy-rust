//! Daisy Patch.SM board helpers.

use stm32h7xx_hal::{
    dac::{DacExt, EnabledUnbuffered, C1, C2},
    gpio::{self, Analog},
    rcc::rec,
    stm32,
    traits::DacOut,
};

const CV_OUT_VOLTS_TO_DAC_CODE: f32 = 819.0;
const DAC_MAX: f32 = 4095.0;

/// Patch.SM CV output channel selector.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CvOutChannel {
    /// Write both CV outputs.
    Both,
    /// Patch.SM C10 / CV_OUT_1 / PA4.
    One,
    /// Patch.SM C1 / CV_OUT_2 / PA5.
    Two,
}

/// DAC-backed Patch.SM CV outputs.
///
/// This mirrors libDaisy's Patch.SM CV-out behavior: PA4 and PA5 are owned by
/// the DAC, outputs are unbuffered, and voltage writes use the 0V..5V eurorack
/// CV-output scale.
pub struct PatchSmCvOut {
    ch1: C1<stm32::DAC, EnabledUnbuffered>,
    ch2: C2<stm32::DAC, EnabledUnbuffered>,
}

impl PatchSmCvOut {
    /// Creates both Patch.SM CV outputs from DAC1 and the PA4/PA5 output pins.
    pub fn new(
        dac: stm32::DAC,
        pins: (gpio::PA4<Analog>, gpio::PA5<Analog>),
        rec: rec::Dac12,
    ) -> Self {
        let (ch1, ch2) = dac.dac(pins, rec);
        Self {
            ch1: ch1.enable_unbuffered(),
            ch2: ch2.enable_unbuffered(),
        }
    }

    /// Writes a voltage to one or both Patch.SM CV outputs.
    ///
    /// `volts` is clamped to the Patch.SM output range, 0V..5V.
    pub fn write(&mut self, channel: CvOutChannel, volts: f32) {
        let raw = Self::volts_to_raw(volts);
        match channel {
            CvOutChannel::Both => {
                self.ch1.set_value(raw);
                self.ch2.set_value(raw);
            }
            CvOutChannel::One => self.ch1.set_value(raw),
            CvOutChannel::Two => self.ch2.set_value(raw),
        }
    }

    /// Converts Patch.SM CV output voltage to a 12-bit DAC code.
    pub fn volts_to_raw(volts: f32) -> u16 {
        let raw = volts.clamp(0.0, 5.0) * CV_OUT_VOLTS_TO_DAC_CODE;
        raw.clamp(0.0, DAC_MAX) as u16
    }
}
