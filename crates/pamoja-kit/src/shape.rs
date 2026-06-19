//! Shaping a reading: ignoring small wiggle around a value.

/// Holds a reading at a center value while it stays within a band, ignoring small wiggle.
///
/// A reading that hovers near a setpoint jitters a little in both directions. Acting on that
/// jitter makes an actuator chatter - a valve or heater switching on and off, "hunting"
/// around the target. A deadband ignores it: while `value` is within `width` of `center` the
/// center is returned unchanged, so nothing downstream reacts; once `value` moves beyond the
/// band it passes through as-is.
///
/// # Arguments
///
/// * `value` - the reading to shape.
/// * `center` - the value the band is centered on.
/// * `width` - the half-width of the band; its magnitude is used, so the band runs from
///   `center - width` to `center + width`.
///
/// # Returns
///
/// `center` when `value` is within `width` of it, otherwise `value` unchanged.
///
/// # Examples
///
/// ```
/// use pamoja_kit::deadband;
///
/// // A setpoint of 20 with a 0.5 deadband ignores small wiggle around it.
/// assert_eq!(deadband(20.2, 20.0, 0.5), 20.0); // within the band: held at center
/// assert_eq!(deadband(21.0, 20.0, 0.5), 21.0); // beyond the band: passes through
/// ```
pub fn deadband(value: f32, center: f32, width: f32) -> f32 {
    let half = if width < 0.0 { -width } else { width };
    let deviation = value - center;
    let distance = if deviation < 0.0 {
        -deviation
    } else {
        deviation
    };
    if distance <= half {
        center
    } else {
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn within_the_band_returns_the_center() {
        assert_eq!(deadband(20.2, 20.0, 0.5), 20.0);
        assert_eq!(deadband(19.8, 20.0, 0.5), 20.0);
    }

    #[test]
    fn beyond_the_band_passes_through() {
        assert_eq!(deadband(21.0, 20.0, 0.5), 21.0);
        assert_eq!(deadband(18.0, 20.0, 0.5), 18.0);
    }

    #[test]
    fn the_band_edge_counts_as_inside() {
        assert_eq!(deadband(20.5, 20.0, 0.5), 20.0);
        assert_eq!(deadband(19.5, 20.0, 0.5), 20.0);
    }

    #[test]
    fn a_negative_width_is_treated_as_its_magnitude() {
        assert_eq!(deadband(20.2, 20.0, -0.5), 20.0);
    }

    #[test]
    fn a_zero_width_holds_only_the_exact_center() {
        assert_eq!(deadband(20.0, 20.0, 0.0), 20.0);
        assert_eq!(deadband(20.1, 20.0, 0.0), 20.1);
    }
}
