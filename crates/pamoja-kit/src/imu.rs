//! Tilt from a three-axis accelerometer.

use core::f64::consts::PI;

use libm::{atan2, sqrt};

fn to_degrees(radians: f64) -> f64 {
    radians * (180.0 / PI)
}

/// Roll and pitch angles, in degrees.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tilt {
    /// Rotation about the forward (x) axis, in degrees, in `[-180.0, 180.0]`.
    pub roll: f64,
    /// Rotation about the right (y) axis, in degrees, in `[-90.0, 90.0]`.
    pub pitch: f64,
}

/// Computes roll and pitch from a three-axis accelerometer reading.
///
/// At rest, gravity tells a three-axis accelerometer which way is down, which fixes the
/// board's tilt. This uses the standard formula (Freescale AN3461): roll is `atan2(ay, az)`
/// and pitch is `atan2(-ax, sqrt(ay^2 + az^2))`, with `atan2` placing each angle in the
/// correct quadrant. The reading's units do not matter (raw counts or g), because only the
/// ratios between axes set the angle and any common scale cancels. It holds while the board
/// is still or moving gently, since it assumes the only acceleration is gravity. Yaw
/// (heading) cannot be found from an accelerometer alone; that needs a magnetometer.
///
/// # Arguments
///
/// * `ax` - acceleration along the x (forward) axis.
/// * `ay` - acceleration along the y (right) axis.
/// * `az` - acceleration along the z (up) axis.
///
/// # Returns
///
/// The [`Tilt`] in degrees.
///
/// # Examples
///
/// ```
/// use pamoja_kit::imu::tilt_from_accel;
///
/// // Board level, 1 g straight down on z: no tilt.
/// let level = tilt_from_accel(0.0, 0.0, 1.0);
/// assert!(level.roll.abs() < 1e-6 && level.pitch.abs() < 1e-6);
///
/// // Tipped fully onto its y axis: 90 degrees of roll.
/// let rolled = tilt_from_accel(0.0, 1.0, 0.0);
/// assert!((rolled.roll - 90.0).abs() < 1e-6);
/// ```
pub fn tilt_from_accel(ax: f64, ay: f64, az: f64) -> Tilt {
    let roll = to_degrees(atan2(ay, az));
    let pitch = to_degrees(atan2(-ax, sqrt(ay * ay + az * az)));
    Tilt { roll, pitch }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_level_board_has_no_tilt() {
        let tilt = tilt_from_accel(0.0, 0.0, 1.0);
        assert!(tilt.roll.abs() < 1e-9);
        assert!(tilt.pitch.abs() < 1e-9);
    }

    #[test]
    fn rolled_onto_the_y_axis_is_ninety_degrees_of_roll() {
        let tilt = tilt_from_accel(0.0, 1.0, 0.0);
        assert!((tilt.roll - 90.0).abs() < 1e-9);
        assert!(tilt.pitch.abs() < 1e-9);
    }

    #[test]
    fn pitched_onto_the_x_axis_is_minus_ninety_pitch() {
        let tilt = tilt_from_accel(1.0, 0.0, 0.0);
        assert!((tilt.pitch + 90.0).abs() < 1e-9);
    }

    #[test]
    fn equal_y_and_z_is_forty_five_degrees_of_roll() {
        let tilt = tilt_from_accel(0.0, 1.0, 1.0);
        assert!((tilt.roll - 45.0).abs() < 1e-9);
    }

    #[test]
    fn the_scale_of_the_reading_does_not_change_the_angle() {
        let g = tilt_from_accel(0.0, 1.0, 1.0);
        let counts = tilt_from_accel(0.0, 1000.0, 1000.0);
        assert!((g.roll - counts.roll).abs() < 1e-9);
    }
}
