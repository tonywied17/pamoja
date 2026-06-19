//! Converting readings between real-world units.
//!
//! Cheap environmental sensors report whatever unit their datasheet chose - a BME280
//! gives pressure in pascals, a thermocouple in degrees Celsius - while the person
//! reading a dashboard thinks in another. These are the exact, named conversions for the
//! units the cookbook uses most, so a conversion is a call with an obvious name rather
//! than a magic constant copied into application code.

/// Pascals in one pound per square inch.
const PASCALS_PER_PSI: f32 = 6894.7573;

/// Converts a temperature from degrees Celsius to degrees Fahrenheit.
///
/// # Arguments
///
/// * `celsius` - a temperature in degrees Celsius.
///
/// # Returns
///
/// The temperature in degrees Fahrenheit, `celsius * 9 / 5 + 32`.
pub fn celsius_to_fahrenheit(celsius: f32) -> f32 {
    celsius * 9.0 / 5.0 + 32.0
}

/// Converts a temperature from degrees Fahrenheit to degrees Celsius.
///
/// # Arguments
///
/// * `fahrenheit` - a temperature in degrees Fahrenheit.
///
/// # Returns
///
/// The temperature in degrees Celsius, `(fahrenheit - 32) * 5 / 9`.
pub fn fahrenheit_to_celsius(fahrenheit: f32) -> f32 {
    (fahrenheit - 32.0) * 5.0 / 9.0
}

/// Converts a temperature from degrees Celsius to kelvin.
///
/// # Arguments
///
/// * `celsius` - a temperature in degrees Celsius.
///
/// # Returns
///
/// The temperature in kelvin, `celsius + 273.15`.
pub fn celsius_to_kelvin(celsius: f32) -> f32 {
    celsius + 273.15
}

/// Converts a temperature from kelvin to degrees Celsius.
///
/// # Arguments
///
/// * `kelvin` - a temperature in kelvin.
///
/// # Returns
///
/// The temperature in degrees Celsius, `kelvin - 273.15`.
pub fn kelvin_to_celsius(kelvin: f32) -> f32 {
    kelvin - 273.15
}

/// Converts a pressure from pascals to hectopascals (millibars).
///
/// # Arguments
///
/// * `pascals` - a pressure in pascals.
///
/// # Returns
///
/// The pressure in hectopascals, the unit weather reports use, `pascals / 100`.
pub fn pascals_to_hectopascals(pascals: f32) -> f32 {
    pascals / 100.0
}

/// Converts a pressure from hectopascals (millibars) to pascals.
///
/// # Arguments
///
/// * `hectopascals` - a pressure in hectopascals.
///
/// # Returns
///
/// The pressure in pascals, `hectopascals * 100`.
pub fn hectopascals_to_pascals(hectopascals: f32) -> f32 {
    hectopascals * 100.0
}

/// Converts a pressure from pascals to kilopascals.
///
/// # Arguments
///
/// * `pascals` - a pressure in pascals.
///
/// # Returns
///
/// The pressure in kilopascals, `pascals / 1000`.
pub fn pascals_to_kilopascals(pascals: f32) -> f32 {
    pascals / 1000.0
}

/// Converts a pressure from kilopascals to pascals.
///
/// # Arguments
///
/// * `kilopascals` - a pressure in kilopascals.
///
/// # Returns
///
/// The pressure in pascals, `kilopascals * 1000`.
pub fn kilopascals_to_pascals(kilopascals: f32) -> f32 {
    kilopascals * 1000.0
}

/// Converts a pressure from pascals to pounds per square inch.
///
/// # Arguments
///
/// * `pascals` - a pressure in pascals.
///
/// # Returns
///
/// The pressure in psi, where one psi is 6894.7573 pascals.
pub fn pascals_to_psi(pascals: f32) -> f32 {
    pascals / PASCALS_PER_PSI
}

/// Converts a pressure from pounds per square inch to pascals.
///
/// # Arguments
///
/// * `psi` - a pressure in pounds per square inch.
///
/// # Returns
///
/// The pressure in pascals, where one psi is 6894.7573 pascals.
pub fn psi_to_pascals(psi: f32) -> f32 {
    psi * PASCALS_PER_PSI
}

/// Converts a fraction in `0.0..=1.0` to a percentage.
///
/// # Arguments
///
/// * `ratio` - a fraction, where `1.0` is the whole.
///
/// # Returns
///
/// The equivalent percentage, `ratio * 100`.
pub fn ratio_to_percent(ratio: f32) -> f32 {
    ratio * 100.0
}

/// Converts a percentage to a fraction in `0.0..=1.0`.
///
/// # Arguments
///
/// * `percent` - a percentage, where `100.0` is the whole.
///
/// # Returns
///
/// The equivalent fraction, `percent / 100`.
pub fn percent_to_ratio(percent: f32) -> f32 {
    percent / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f32, b: f32, tol: f32) -> bool {
        (a - b).abs() < tol
    }

    #[test]
    fn temperature_reference_points() {
        assert!(approx(celsius_to_fahrenheit(100.0), 212.0, 1e-3));
        assert!(approx(celsius_to_fahrenheit(0.0), 32.0, 1e-3));
        assert!(approx(celsius_to_fahrenheit(37.0), 98.6, 1e-2));
        // -40 is the point where the Celsius and Fahrenheit scales coincide.
        assert!(approx(celsius_to_fahrenheit(-40.0), -40.0, 1e-3));
        assert!(approx(fahrenheit_to_celsius(212.0), 100.0, 1e-3));
        assert!(approx(celsius_to_kelvin(0.0), 273.15, 1e-2));
        assert!(approx(kelvin_to_celsius(273.15), 0.0, 1e-2));
    }

    #[test]
    fn temperature_round_trips() {
        assert!(approx(fahrenheit_to_celsius(celsius_to_fahrenheit(21.0)), 21.0, 1e-3));
        assert!(approx(kelvin_to_celsius(celsius_to_kelvin(21.0)), 21.0, 1e-3));
    }

    #[test]
    fn pressure_reference_points() {
        // One standard atmosphere expressed four ways.
        assert!(approx(pascals_to_hectopascals(101325.0), 1013.25, 1e-1));
        assert!(approx(pascals_to_kilopascals(101325.0), 101.325, 1e-2));
        assert!(approx(pascals_to_psi(101325.0), 14.6959, 1e-2));
        assert!(approx(hectopascals_to_pascals(1013.25), 101325.0, 1.0));
        assert!(approx(psi_to_pascals(1.0), 6894.7573, 1e-1));
    }

    #[test]
    fn percent_helpers() {
        assert!(approx(ratio_to_percent(0.25), 25.0, 1e-4));
        assert!(approx(percent_to_ratio(25.0), 0.25, 1e-4));
        assert!(approx(percent_to_ratio(ratio_to_percent(0.6)), 0.6, 1e-4));
    }
}
