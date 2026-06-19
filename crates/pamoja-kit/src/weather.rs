//! Humidity-derived values: the dew point.

use libm::log;

/// Magnus coefficient (dimensionless), for the range about -45 to 60 C.
const MAGNUS_B: f64 = 17.62;
/// Magnus coefficient, in degrees Celsius.
const MAGNUS_C: f64 = 243.12;

/// Computes the dew point from temperature and relative humidity (Magnus formula).
///
/// The dew point is the temperature to which air must cool for its moisture to begin to
/// condense; it is the practical signal behind condensation, fog, and frost. This uses the
/// Magnus-Tetens approximation with the WMO coefficients (b = 17.62, c = 243.12 C), accurate
/// from roughly -45 to 60 C: with `gamma = ln(rh / 100) + b * t / (c + t)`, the dew point is
/// `c * gamma / (b - gamma)`. A dew point at or below 0 C means any condensation forms as
/// frost, the basis of an overnight frost warning for a crop.
///
/// # Arguments
///
/// * `celsius` - the air temperature in degrees Celsius.
/// * `humidity_percent` - the relative humidity in percent, in `(0, 100]`. A value at or
///   below zero is treated as a tiny positive value so the logarithm stays defined.
///
/// # Returns
///
/// The dew point in degrees Celsius.
///
/// # Examples
///
/// ```
/// use pamoja_kit::weather::dew_point;
///
/// // 20 C air at 50% relative humidity dews near 9.3 C.
/// assert!((dew_point(20.0, 50.0) - 9.3).abs() < 0.2);
///
/// // Saturated air: the dew point equals the temperature.
/// assert!((dew_point(15.0, 100.0) - 15.0).abs() < 1e-6);
/// ```
pub fn dew_point(celsius: f64, humidity_percent: f64) -> f64 {
    let humidity = if humidity_percent <= 0.0 {
        0.0001
    } else {
        humidity_percent
    };
    let gamma = log(humidity / 100.0) + MAGNUS_B * celsius / (MAGNUS_C + celsius);
    MAGNUS_C * gamma / (MAGNUS_B - gamma)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_a_worked_example() {
        // 20 C at 50% RH dews near 9.3 C with the Magnus coefficients b=17.62, c=243.12.
        assert!((dew_point(20.0, 50.0) - 9.3).abs() < 0.2);
    }

    #[test]
    fn saturated_air_dews_at_the_temperature() {
        assert!((dew_point(15.0, 100.0) - 15.0).abs() < 1e-6);
        assert!((dew_point(-3.0, 100.0) + 3.0).abs() < 1e-6);
    }

    #[test]
    fn lower_humidity_means_a_lower_dew_point() {
        let humid = dew_point(25.0, 80.0);
        let dry = dew_point(25.0, 30.0);
        assert!(dry < humid);
    }

    #[test]
    fn a_frost_risk_shows_as_a_dew_point_below_zero() {
        // Cold, fairly dry air dews below freezing.
        assert!(dew_point(2.0, 60.0) < 0.0);
    }
}
