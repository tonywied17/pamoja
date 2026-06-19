//! Differential-drive wheel kinematics.

/// Converts between a robot's motion and its two wheel speeds (differential drive).
///
/// A differential-drive robot steers by spinning its left and right wheels at different
/// speeds. This converts both ways: [`wheel_speeds`](DiffDrive::wheel_speeds) turns a desired
/// forward speed and turn rate into the wheel speeds to command (inverse kinematics), and
/// [`body_motion`](DiffDrive::body_motion) turns measured wheel speeds back into the robot's
/// forward speed and turn rate (forward kinematics). The one parameter is the track: the
/// distance between the wheels.
///
/// # Examples
///
/// ```
/// use pamoja_kit::DiffDrive;
///
/// let drive = DiffDrive::new(0.5); // wheels 0.5 apart
/// // Drive straight: both wheels turn at the forward speed.
/// assert_eq!(drive.wheel_speeds(1.0, 0.0), (1.0, 1.0));
/// // Spin in place: the wheels turn opposite, each at turn rate times half the track.
/// assert_eq!(drive.wheel_speeds(0.0, 2.0), (-0.5, 0.5));
/// ```
#[derive(Clone, Copy, Debug)]
pub struct DiffDrive {
    track: f32,
}

impl DiffDrive {
    /// Creates a model for wheels `track` apart.
    ///
    /// # Arguments
    ///
    /// * `track` - the distance between the left and right wheels; its magnitude is used.
    ///
    /// # Returns
    ///
    /// The kinematics model.
    pub fn new(track: f32) -> Self {
        Self {
            track: magnitude(track),
        }
    }

    /// Returns the `(left, right)` wheel speeds for a desired body motion.
    ///
    /// # Arguments
    ///
    /// * `linear` - the forward speed.
    /// * `angular` - the turn rate, positive turning toward the left (counter-clockwise).
    ///
    /// # Returns
    ///
    /// `(left, right)`, where `left = linear - angular * track / 2` and
    /// `right = linear + angular * track / 2`.
    pub fn wheel_speeds(&self, linear: f32, angular: f32) -> (f32, f32) {
        let half = angular * self.track / 2.0;
        (linear - half, linear + half)
    }

    /// Returns the body `(linear, angular)` motion for measured wheel speeds.
    ///
    /// # Arguments
    ///
    /// * `left` - the left wheel speed.
    /// * `right` - the right wheel speed.
    ///
    /// # Returns
    ///
    /// `(linear, angular)`, where `linear = (right + left) / 2` and
    /// `angular = (right - left) / track`. Angular is zero when the track is zero.
    pub fn body_motion(&self, left: f32, right: f32) -> (f32, f32) {
        let linear = (right + left) / 2.0;
        let angular = if self.track == 0.0 {
            0.0
        } else {
            (right - left) / self.track
        };
        (linear, angular)
    }
}

// `f32::abs` lives in `std`, so this `no_std` crate takes the magnitude by hand.
fn magnitude(value: f32) -> f32 {
    if value < 0.0 {
        -value
    } else {
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn driving_straight_turns_both_wheels_equally() {
        let drive = DiffDrive::new(0.5);
        assert_eq!(drive.wheel_speeds(1.0, 0.0), (1.0, 1.0));
    }

    #[test]
    fn spinning_in_place_turns_the_wheels_opposite() {
        let drive = DiffDrive::new(0.5);
        assert_eq!(drive.wheel_speeds(0.0, 2.0), (-0.5, 0.5));
    }

    #[test]
    fn body_motion_inverts_wheel_speeds() {
        let drive = DiffDrive::new(0.5);
        assert_eq!(drive.body_motion(1.0, 1.0), (1.0, 0.0)); // straight
        assert_eq!(drive.body_motion(-0.5, 0.5), (0.0, 2.0)); // spinning
    }

    #[test]
    fn the_two_directions_round_trip() {
        let drive = DiffDrive::new(0.42);
        let (left, right) = drive.wheel_speeds(1.3, -0.7);
        let (linear, angular) = drive.body_motion(left, right);
        assert!((linear - 1.3).abs() < 1e-6);
        assert!((angular + 0.7).abs() < 1e-6);
    }

    #[test]
    fn a_zero_track_reports_no_rotation() {
        let drive = DiffDrive::new(0.0);
        assert_eq!(drive.body_motion(1.0, 2.0), (1.5, 0.0));
    }
}
