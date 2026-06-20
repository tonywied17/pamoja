//! Texas Instruments INA219 high-side current, voltage, and power monitor.
//!
//! The INA219 measures the voltage across a shunt resistor and the bus voltage, and,
//! once its calibration register is programmed, computes current and power on the
//! chip. This module builds the calibration value and decodes each register into a
//! physical quantity, following the datasheet's equations and its worked design
//! example, so a solar-battery or microgrid node reads amps and watts directly.
//!
//! Currents, voltages, and powers are returned in integer micro-units (microvolts,
//! microamps, microwatts) so the conversions stay exact without floating point.

/// The INA219 register addresses.
pub mod register {
    /// Configuration register: bus-voltage range, gain, ADC settings, and mode.
    pub const CONFIGURATION: u8 = 0x00;
    /// Shunt voltage register, signed, 10 µV per count.
    pub const SHUNT_VOLTAGE: u8 = 0x01;
    /// Bus voltage register, value in bits 15:3, 4 mV per count.
    pub const BUS_VOLTAGE: u8 = 0x02;
    /// Power register, scaled by the calibration register.
    pub const POWER: u8 = 0x03;
    /// Current register, scaled by the calibration register.
    pub const CURRENT: u8 = 0x04;
    /// Calibration register, sets the current and power scale.
    pub const CALIBRATION: u8 = 0x05;
}

/// The power-on value of the configuration register (0x399F): 32 V bus range, gain
/// /8, 12-bit ADCs, and continuous shunt-and-bus conversion.
pub const CONFIG_RESET: u16 = 0x399F;

/// Computes the calibration register value for a chosen current resolution and shunt.
///
/// This is the datasheet's calibration equation, `Cal = trunc(0.04096 / (Current_LSB
/// * R_shunt))`, expressed in integer micro-units: with the current LSB in microamps
/// and the shunt in milliohms, the fixed `0.04096` becomes `40_960_000`.
///
/// # Arguments
///
/// * `current_lsb_microamps` - the amps-per-count the current register should carry.
/// * `shunt_milliohms` - the shunt resistor value, in milliohms.
///
/// # Returns
///
/// The 16-bit value to program into the calibration register. Returns `0` if either
/// argument is `0`, which is the chip's own uncalibrated state.
pub fn calibration(current_lsb_microamps: u32, shunt_milliohms: u32) -> u16 {
    let denominator = current_lsb_microamps.saturating_mul(shunt_milliohms);
    if denominator == 0 {
        return 0;
    }
    (40_960_000 / denominator) as u16
}

/// Returns the smallest current LSB, in microamps, that still spans a full-scale
/// current.
///
/// The current register is 15 bits of magnitude, so the minimum resolution is the
/// maximum expected current divided by 32768, rounded up to the next whole microamp.
/// The datasheet then rounds this up further to a convenient round number.
///
/// # Arguments
///
/// * `max_expected_microamps` - the largest current the application will measure.
///
/// # Returns
///
/// The minimum current LSB in microamps.
pub fn minimum_current_lsb_microamps(max_expected_microamps: u32) -> u32 {
    max_expected_microamps.div_ceil(32_768)
}

/// Decodes the shunt voltage register to microvolts.
///
/// # Arguments
///
/// * `raw` - the signed shunt voltage register.
///
/// # Returns
///
/// The shunt voltage in microvolts, at 10 µV per count.
pub fn shunt_microvolts(raw: i16) -> i32 {
    raw as i32 * 10
}

/// Decodes the bus voltage register to millivolts.
///
/// The voltage occupies bits 15:3, so the register is shifted right by three before
/// scaling by the 4 mV LSB; the low bits are the conversion-ready and overflow flags.
///
/// # Arguments
///
/// * `raw` - the bus voltage register.
///
/// # Returns
///
/// The bus voltage in millivolts.
pub fn bus_millivolts(raw: u16) -> u32 {
    (raw >> 3) as u32 * 4
}

/// Returns whether the bus voltage register's conversion-ready (CNVR) flag is set.
///
/// # Arguments
///
/// * `raw` - the bus voltage register.
///
/// # Returns
///
/// `true` if a conversion has completed and the data is ready to read.
pub fn conversion_ready(raw: u16) -> bool {
    raw & 0x0002 != 0
}

/// Returns whether the bus voltage register's math-overflow (OVF) flag is set.
///
/// # Arguments
///
/// * `raw` - the bus voltage register.
///
/// # Returns
///
/// `true` if the power or current calculation overflowed and the readings are invalid.
pub fn math_overflow(raw: u16) -> bool {
    raw & 0x0001 != 0
}

/// Decodes the current register to microamps for a given current LSB.
///
/// # Arguments
///
/// * `raw` - the signed current register.
/// * `current_lsb_microamps` - the current LSB the calibration register was set for.
///
/// # Returns
///
/// The current in microamps.
pub fn current_microamps(raw: i16, current_lsb_microamps: u32) -> i32 {
    raw as i32 * current_lsb_microamps as i32
}

/// Decodes the power register to microwatts for a given current LSB.
///
/// The power LSB is fixed by the datasheet at twenty times the current LSB.
///
/// # Arguments
///
/// * `raw` - the power register.
/// * `current_lsb_microamps` - the current LSB the calibration register was set for.
///
/// # Returns
///
/// The power in microwatts.
pub fn power_microwatts(raw: u16, current_lsb_microamps: u32) -> u32 {
    raw as u32 * (20 * current_lsb_microamps)
}

#[cfg(test)]
mod tests {
    use super::*;

    // The datasheet's worked design example (Table 8): max expected current 15 A,
    // shunt 2 mΩ, current LSB rounded to 1 mA/bit, bus range 16 V.
    const CURRENT_LSB: u32 = 1_000; // 1 mA in microamps

    #[test]
    fn calibration_matches_the_datasheet_example() {
        // Cal = trunc(0.04096 / (0.001 A * 0.002 Ω)) = 20480 = 0x5000.
        assert_eq!(calibration(CURRENT_LSB, 2), 20_480);
        assert_eq!(calibration(CURRENT_LSB, 2), 0x5000);
    }

    #[test]
    fn minimum_current_lsb_matches_the_datasheet_example() {
        // 15 A / 32768 = 457.76 µA, computed up to the next whole microamp.
        assert_eq!(minimum_current_lsb_microamps(15_000_000), 458);
    }

    #[test]
    fn shunt_register_decodes_per_the_datasheet() {
        // Table 8: shunt register 0x07D0 = 2000 → 20 mV.
        assert_eq!(shunt_microvolts(0x07D0), 20_000);
        // Negative full scale at gain /8: -320 mV is register 0x8300 (Figure 20).
        assert_eq!(shunt_microvolts(i16::from_be_bytes([0x83, 0x00])), -320_000);
    }

    #[test]
    fn bus_register_decodes_per_the_datasheet() {
        // Table 8: bus register 0x5D98 → shifted 0x0BB3 = 2995 → 11.98 V.
        assert_eq!(bus_millivolts(0x5D98), 11_980);
        assert!(!conversion_ready(0x5D98));
        assert!(!math_overflow(0x5D98));
        // A reading with both status flags set.
        assert!(conversion_ready(0x1F43));
        assert!(math_overflow(0x1F43));
    }

    #[test]
    fn current_register_decodes_to_the_example_load() {
        // Table 8: current register 0x2710 = 10000 → 10.0 A.
        assert_eq!(current_microamps(0x2710, CURRENT_LSB), 10_000_000);
    }

    #[test]
    fn power_register_decodes_to_the_example_load() {
        // Table 8: power register 0x1766 = 5990 → 119.8 W (power LSB 20 mW).
        assert_eq!(power_microwatts(0x1766, CURRENT_LSB), 119_800_000);
    }

    #[test]
    fn an_uncalibrated_request_returns_zero() {
        assert_eq!(calibration(0, 2), 0);
        assert_eq!(calibration(CURRENT_LSB, 0), 0);
    }
}
