//! Easing a value toward a target at a limited rate.

/// Moves a value toward a target by at most a fixed step each update.
///
/// Commanding an actuator straight to a new value can be harsh: a motor lurches, a valve
/// slams, a lamp jumps. A [`Ramp`] limits how fast the commanded value may change, easing it
/// toward the target by at most `max_step` per update and snapping to the target once it is
/// within a step. It is the slew-rate limiter behind a smooth start and stop.
///
/// # Examples
///
/// ```
/// use pamoja_kit::Ramp;
///
/// // Start at 0, move at most 2 per step, aim for 5.
/// let mut ramp = Ramp::new(0.0, 2.0);
/// assert_eq!(ramp.update(5.0), 2.0);
/// assert_eq!(ramp.update(5.0), 4.0);
/// assert_eq!(ramp.update(5.0), 5.0); // within a step: snaps to the target
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Ramp {
    value: f32,
    max_step: f32,
}

impl Ramp {
    /// Creates a ramp starting at `start` that moves at most `max_step` per update.
    ///
    /// # Arguments
    ///
    /// * `start` - the initial value.
    /// * `max_step` - the largest change allowed per update; its magnitude is used.
    ///
    /// # Returns
    ///
    /// A ramp resting at `start`.
    pub fn new(start: f32, max_step: f32) -> Self {
        Self {
            value: start,
            max_step: magnitude(max_step),
        }
    }

    /// Moves toward `target` by at most the step and returns the new value.
    ///
    /// # Arguments
    ///
    /// * `target` - the value being approached.
    ///
    /// # Returns
    ///
    /// The value after one limited step, equal to `target` once within a step of it.
    pub fn update(&mut self, target: f32) -> f32 {
        let delta = target - self.value;
        if delta > self.max_step {
            self.value += self.max_step;
        } else if delta < -self.max_step {
            self.value -= self.max_step;
        } else {
            self.value = target;
        }
        self.value
    }

    /// Returns the current value.
    pub fn value(&self) -> f32 {
        self.value
    }

    /// Jumps directly to `value`, bypassing the rate limit.
    ///
    /// # Arguments
    ///
    /// * `value` - the new value.
    pub fn set(&mut self, value: f32) {
        self.value = value;
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
    fn it_climbs_toward_a_higher_target() {
        let mut ramp = Ramp::new(0.0, 2.0);
        assert_eq!(ramp.update(5.0), 2.0);
        assert_eq!(ramp.update(5.0), 4.0);
        assert_eq!(ramp.update(5.0), 5.0); // snaps within a step
        assert_eq!(ramp.update(5.0), 5.0); // holds at the target
    }

    #[test]
    fn it_falls_toward_a_lower_target() {
        let mut ramp = Ramp::new(10.0, 3.0);
        assert_eq!(ramp.update(0.0), 7.0);
        assert_eq!(ramp.update(0.0), 4.0);
        assert_eq!(ramp.update(0.0), 1.0);
        assert_eq!(ramp.update(0.0), 0.0);
    }

    #[test]
    fn a_negative_step_is_treated_as_its_magnitude() {
        let mut ramp = Ramp::new(0.0, -2.0);
        assert_eq!(ramp.update(10.0), 2.0);
    }

    #[test]
    fn set_jumps_past_the_limit() {
        let mut ramp = Ramp::new(0.0, 1.0);
        ramp.set(100.0);
        assert_eq!(ramp.value(), 100.0);
    }
}
