//! Bosch BME280 temperature, pressure, and humidity sensor.
//!
//! The BME280 ships raw, uncompensated readings plus a block of per-chip calibration
//! coefficients; the real measurement only appears after running those raw values
//! through Bosch's compensation formulas. Those formulas are the classic place a
//! from-memory port goes subtly wrong, so the integer arithmetic here is ported from
//! Bosch's published reference code, and the tests cross-check it against the
//! reference floating-point form and the datasheet's worked temperature example.
//!
//! A caller reads the calibration registers once with [`Calibration::from_registers`],
//! then reads the data registers each cycle into a [`RawMeasurement`] and calls
//! [`Calibration::compensate`].

/// The I2C address with the SDO pin tied low.
pub const I2C_ADDRESS_PRIMARY: u8 = 0x76;
/// The I2C address with the SDO pin tied high.
pub const I2C_ADDRESS_SECONDARY: u8 = 0x77;
/// The value the chip-id register (0xD0) returns for a BME280.
pub const CHIP_ID: u8 = 0x60;

/// The BME280 register addresses used to read calibration and measurements.
pub mod register {
    /// Chip-id register; reads [`super::CHIP_ID`] for a BME280.
    pub const CHIP_ID: u8 = 0xD0;
    /// Soft-reset register.
    pub const RESET: u8 = 0xE0;
    /// First of the 26 temperature and pressure calibration bytes (0x88..=0xA1).
    pub const CALIB_TEMP_PRESS: u8 = 0x88;
    /// First of the 7 humidity calibration bytes (0xE1..=0xE7).
    pub const CALIB_HUMIDITY: u8 = 0xE1;
    /// First of the 8 burst-read data bytes: pressure, temperature, humidity
    /// (0xF7..=0xFE).
    pub const DATA: u8 = 0xF7;
}

/// The per-chip calibration coefficients read from the sensor's calibration registers.
///
/// These are factory-programmed and constant for a given device, so they are read
/// once at start-up and reused for every measurement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Calibration {
    dig_t1: u16,
    dig_t2: i16,
    dig_t3: i16,
    dig_p1: u16,
    dig_p2: i16,
    dig_p3: i16,
    dig_p4: i16,
    dig_p5: i16,
    dig_p6: i16,
    dig_p7: i16,
    dig_p8: i16,
    dig_p9: i16,
    dig_h1: u8,
    dig_h2: i16,
    dig_h3: u8,
    dig_h4: i16,
    dig_h5: i16,
    dig_h6: i8,
}

impl Calibration {
    /// Parses the calibration coefficients from the two register blocks.
    ///
    /// The split-and-shifted packing of `dig_h4` and `dig_h5`, which share a byte, is
    /// done exactly as the datasheet specifies.
    ///
    /// # Arguments
    ///
    /// * `temp_press` - the 26 bytes read from `0x88..=0xA1`.
    /// * `humidity` - the 7 bytes read from `0xE1..=0xE7`.
    ///
    /// # Returns
    ///
    /// The decoded calibration.
    pub fn from_registers(temp_press: &[u8; 26], humidity: &[u8; 7]) -> Calibration {
        let tp = temp_press;
        let h = humidity;
        Calibration {
            dig_t1: u16::from_le_bytes([tp[0], tp[1]]),
            dig_t2: i16::from_le_bytes([tp[2], tp[3]]),
            dig_t3: i16::from_le_bytes([tp[4], tp[5]]),
            dig_p1: u16::from_le_bytes([tp[6], tp[7]]),
            dig_p2: i16::from_le_bytes([tp[8], tp[9]]),
            dig_p3: i16::from_le_bytes([tp[10], tp[11]]),
            dig_p4: i16::from_le_bytes([tp[12], tp[13]]),
            dig_p5: i16::from_le_bytes([tp[14], tp[15]]),
            dig_p6: i16::from_le_bytes([tp[16], tp[17]]),
            dig_p7: i16::from_le_bytes([tp[18], tp[19]]),
            dig_p8: i16::from_le_bytes([tp[20], tp[21]]),
            dig_p9: i16::from_le_bytes([tp[22], tp[23]]),
            dig_h1: tp[25],
            dig_h2: i16::from_le_bytes([h[0], h[1]]),
            dig_h3: h[2],
            dig_h4: ((h[3] as i8 as i16) * 16) | (h[4] & 0x0F) as i16,
            dig_h5: ((h[5] as i8 as i16) * 16) | (h[4] >> 4) as i16,
            dig_h6: h[6] as i8,
        }
    }

    /// Compensates a raw measurement into temperature, pressure, and humidity.
    ///
    /// Temperature is computed first because its intermediate `t_fine` term feeds the
    /// pressure and humidity formulas, exactly as the reference code shares it.
    ///
    /// # Arguments
    ///
    /// * `raw` - the uncompensated readings from the data registers.
    ///
    /// # Returns
    ///
    /// The compensated [`Measurement`].
    pub fn compensate(&self, raw: &RawMeasurement) -> Measurement {
        let t_fine = self.t_fine(raw.temperature);
        Measurement {
            temperature_centi_celsius: compensate_temperature(t_fine),
            pressure_centi_pascals: self.compensate_pressure(raw.pressure, t_fine),
            humidity_q22_10: self.compensate_humidity(raw.humidity, t_fine),
        }
    }

    // The shared fine-temperature term, in the reference code's fixed-point form.
    fn t_fine(&self, adc_t: i32) -> i32 {
        let var1 = ((adc_t / 8) - (self.dig_t1 as i32 * 2)) * self.dig_t2 as i32 / 2048;
        let near = (adc_t / 16) - self.dig_t1 as i32;
        let var2 = (((near * near) / 4096) * self.dig_t3 as i32) / 16384;
        var1 + var2
    }

    // Pressure in hundredths of a pascal, via the 64-bit reference path.
    fn compensate_pressure(&self, adc_p: i32, t_fine: i32) -> u32 {
        let mut var1 = t_fine as i64 - 128000;
        let mut var2 = var1 * var1 * self.dig_p6 as i64;
        var2 += (var1 * self.dig_p5 as i64) * 131072;
        var2 += self.dig_p4 as i64 * 34359738368;
        var1 = (var1 * var1 * self.dig_p3 as i64) / 256 + (var1 * self.dig_p2 as i64 * 4096);
        var1 = (140737488355328 + var1) * self.dig_p1 as i64 / 8589934592;
        if var1 == 0 {
            return 3_000_000;
        }
        let mut var4 = 1048576 - adc_p as i64;
        var4 = (((var4 * 2147483648) - var2) * 3125) / var1;
        var1 = (self.dig_p9 as i64 * (var4 / 8192) * (var4 / 8192)) / 33554432;
        var2 = (self.dig_p8 as i64 * var4) / 524288;
        var4 = ((var4 + var1 + var2) / 256) + (self.dig_p7 as i64 * 16);
        let pressure = ((var4 / 2) * 100) / 128;
        pressure.clamp(3_000_000, 11_000_000) as u32
    }

    // Humidity in Q22.10 fixed-point (units of 1/1024 %), via the reference path.
    fn compensate_humidity(&self, adc_h: i32, t_fine: i32) -> u32 {
        let var1 = t_fine - 76800;
        let var2 = adc_h * 16384;
        let var3 = self.dig_h4 as i32 * 1048576;
        let var4 = self.dig_h5 as i32 * var1;
        let var5 = (((var2 - var3) - var4) + 16384) / 32768;
        let var2 = (var1 * self.dig_h6 as i32) / 1024;
        let var3 = (var1 * self.dig_h3 as i32) / 2048;
        let var4 = ((var2 * (var3 + 32768)) / 1024) + 2097152;
        let var2 = ((var4 * self.dig_h2 as i32) + 8192) / 16384;
        let var3 = var5 * var2;
        let var4 = ((var3 / 32768) * (var3 / 32768)) / 128;
        let var5 = (var3 - ((var4 * self.dig_h1 as i32) / 16)).clamp(0, 419430400);
        ((var5 / 4096) as u32).min(102400)
    }
}

// The integer temperature in hundredths of a degree Celsius, clamped to range.
fn compensate_temperature(t_fine: i32) -> i32 {
    ((t_fine * 5 + 128) / 256).clamp(-4000, 8500)
}

/// The raw, uncompensated readings from the BME280 data registers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RawMeasurement {
    /// The 20-bit uncompensated temperature.
    pub temperature: i32,
    /// The 20-bit uncompensated pressure.
    pub pressure: i32,
    /// The 16-bit uncompensated humidity.
    pub humidity: i32,
}

impl RawMeasurement {
    /// Parses the eight data-register bytes burst-read from `0xF7..=0xFE`.
    ///
    /// The order on the wire is pressure (20 bits), temperature (20 bits), then
    /// humidity (16 bits), each most significant byte first.
    ///
    /// # Arguments
    ///
    /// * `data` - the eight bytes read from the data registers.
    ///
    /// # Returns
    ///
    /// The unpacked raw measurement.
    pub fn from_registers(data: &[u8; 8]) -> RawMeasurement {
        let pressure =
            (i32::from(data[0]) << 12) | (i32::from(data[1]) << 4) | (i32::from(data[2]) >> 4);
        let temperature =
            (i32::from(data[3]) << 12) | (i32::from(data[4]) << 4) | (i32::from(data[5]) >> 4);
        let humidity = (i32::from(data[6]) << 8) | i32::from(data[7]);
        RawMeasurement {
            temperature,
            pressure,
            humidity,
        }
    }
}

/// A compensated BME280 measurement.
///
/// The fields are the exact integer outputs of the compensation formulas; the
/// methods present them in conventional units.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Measurement {
    /// Temperature in hundredths of a degree Celsius.
    pub temperature_centi_celsius: i32,
    /// Pressure in hundredths of a pascal.
    pub pressure_centi_pascals: u32,
    /// Relative humidity in Q22.10 fixed-point, that is units of 1/1024 percent.
    pub humidity_q22_10: u32,
}

impl Measurement {
    /// Returns the temperature in degrees Celsius.
    pub fn celsius(&self) -> f32 {
        self.temperature_centi_celsius as f32 / 100.0
    }

    /// Returns the pressure in pascals.
    pub fn pascals(&self) -> u32 {
        self.pressure_centi_pascals / 100
    }

    /// Returns the pressure in hectopascals (millibars).
    pub fn hectopascals(&self) -> f32 {
        self.pressure_centi_pascals as f32 / 10_000.0
    }

    /// Returns the relative humidity in percent.
    pub fn relative_humidity_percent(&self) -> f32 {
        self.humidity_q22_10 as f32 / 1024.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A representative, internally consistent calibration set, used to exercise the
    // formulas across a sweep of raw codes.
    fn sample_calibration() -> Calibration {
        Calibration {
            dig_t1: 28485,
            dig_t2: 26735,
            dig_t3: 50,
            dig_p1: 37190,
            dig_p2: -10646,
            dig_p3: 3024,
            dig_p4: 7758,
            dig_p5: -120,
            dig_p6: -7,
            dig_p7: 9900,
            dig_p8: -10230,
            dig_p9: 4285,
            dig_h1: 75,
            dig_h2: 354,
            dig_h3: 0,
            dig_h4: 339,
            dig_h5: 50,
            dig_h6: 30,
        }
    }

    #[test]
    fn temperature_matches_the_datasheet_worked_example() {
        // Bosch's published worked example: dig_t1 = 27504, dig_t2 = 26435,
        // dig_t3 = -1000, adc_t = 519888 gives T = 25.08 °C (2508 in hundredths). The
        // shared t_fine intermediate lands at 128423 with the current integer formula,
        // one count off the 128422 quoted from the older BMP280 example; the published
        // result, the 25.08 °C output, is what this anchors to.
        let calib = Calibration {
            dig_t1: 27504,
            dig_t2: 26435,
            dig_t3: -1000,
            ..sample_calibration()
        };
        let t_fine = calib.t_fine(519888);
        assert!((t_fine - 128422).abs() <= 1, "t_fine {t_fine}");
        assert_eq!(compensate_temperature(t_fine), 2508);
    }

    #[test]
    fn integer_compensation_tracks_the_floating_point_reference() {
        let calib = sample_calibration();
        // Sweep a range of raw codes and require the integer path to agree with the
        // structurally different floating-point reference to within rounding.
        for &adc_t in &[400_000, 500_000, 519_888, 540_000] {
            let t_fine = calib.t_fine(adc_t);
            let t_int = compensate_temperature(t_fine);
            let t_ref = reference_temperature(&calib, adc_t);
            assert!(
                ((t_int as f64) / 100.0 - t_ref).abs() < 0.02,
                "temperature {t_int} vs {t_ref}"
            );

            for &adc_p in &[300_000, 415_148, 512_000] {
                let p_int = calib.compensate_pressure(adc_p, t_fine);
                let p_ref = reference_pressure(&calib, adc_p, adc_t);
                assert!(
                    ((p_int as f64) / 100.0 - p_ref).abs() < 2.0,
                    "pressure {p_int} (centi-Pa) vs {p_ref} Pa"
                );
            }

            for &adc_h in &[20_000, 30_000, 45_000] {
                let h_int = calib.compensate_humidity(adc_h, t_fine);
                let h_ref = reference_humidity(&calib, adc_h, adc_t);
                assert!(
                    ((h_int as f64) / 1024.0 - h_ref).abs() < 0.05,
                    "humidity {h_int} (Q22.10) vs {h_ref} %"
                );
            }
        }
    }

    #[test]
    fn raw_measurement_unpacks_the_data_registers() {
        // Pressure 0x53D0E, temperature 0x81D90, humidity 0x6E62.
        let data = [0x53, 0xD0, 0xE0, 0x81, 0xD9, 0x00, 0x6E, 0x62];
        let raw = RawMeasurement::from_registers(&data);
        assert_eq!(raw.pressure, 0x5_3D0E);
        assert_eq!(raw.temperature, 0x8_1D90);
        assert_eq!(raw.humidity, 0x6E62);
    }

    #[test]
    fn measurement_unit_helpers_convert_correctly() {
        let measurement = Measurement {
            temperature_centi_celsius: 2508,
            pressure_centi_pascals: 10_065_300,
            humidity_q22_10: 47_104,
        };
        assert!((measurement.celsius() - 25.08).abs() < 0.001);
        assert_eq!(measurement.pascals(), 100_653);
        assert!((measurement.hectopascals() - 1006.53).abs() < 0.01);
        assert!((measurement.relative_humidity_percent() - 46.0).abs() < 0.001);
    }

    // The reference floating-point compensation from Bosch's published code, used only
    // to validate the integer path above.
    fn reference_temperature(c: &Calibration, adc_t: i32) -> f64 {
        let var1 = (adc_t as f64 / 16384.0 - c.dig_t1 as f64 / 1024.0) * c.dig_t2 as f64;
        let var2 = {
            let v = adc_t as f64 / 131072.0 - c.dig_t1 as f64 / 8192.0;
            v * v * c.dig_t3 as f64
        };
        ((var1 + var2) / 5120.0).clamp(-40.0, 85.0)
    }

    fn reference_t_fine(c: &Calibration, adc_t: i32) -> f64 {
        let var1 = (adc_t as f64 / 16384.0 - c.dig_t1 as f64 / 1024.0) * c.dig_t2 as f64;
        let var2 = {
            let v = adc_t as f64 / 131072.0 - c.dig_t1 as f64 / 8192.0;
            v * v * c.dig_t3 as f64
        };
        var1 + var2
    }

    fn reference_pressure(c: &Calibration, adc_p: i32, adc_t: i32) -> f64 {
        let t_fine = reference_t_fine(c, adc_t);
        let mut var1 = (t_fine / 2.0) - 64000.0;
        let mut var2 = var1 * var1 * c.dig_p6 as f64 / 32768.0;
        var2 += var1 * c.dig_p5 as f64 * 2.0;
        var2 = (var2 / 4.0) + (c.dig_p4 as f64 * 65536.0);
        let var3 = c.dig_p3 as f64 * var1 * var1 / 524288.0;
        var1 = (var3 + c.dig_p2 as f64 * var1) / 524288.0;
        var1 = (1.0 + var1 / 32768.0) * c.dig_p1 as f64;
        if var1 <= 0.0 {
            return 30000.0;
        }
        let mut pressure = 1048576.0 - adc_p as f64;
        pressure = (pressure - (var2 / 4096.0)) * 6250.0 / var1;
        var1 = c.dig_p9 as f64 * pressure * pressure / 2147483648.0;
        var2 = pressure * c.dig_p8 as f64 / 32768.0;
        pressure += (var1 + var2 + c.dig_p7 as f64) / 16.0;
        pressure.clamp(30000.0, 110000.0)
    }

    fn reference_humidity(c: &Calibration, adc_h: i32, adc_t: i32) -> f64 {
        let t_fine = reference_t_fine(c, adc_t);
        let var1 = t_fine - 76800.0;
        let var2 = c.dig_h4 as f64 * 64.0 + (c.dig_h5 as f64 / 16384.0) * var1;
        let var3 = adc_h as f64 - var2;
        let var4 = c.dig_h2 as f64 / 65536.0;
        let var5 = 1.0 + (c.dig_h3 as f64 / 67108864.0) * var1;
        let var6 = 1.0 + (c.dig_h6 as f64 / 67108864.0) * var1 * var5;
        let var6 = var3 * var4 * (var5 * var6);
        (var6 * (1.0 - c.dig_h1 as f64 * var6 / 524288.0)).clamp(0.0, 100.0)
    }
}
