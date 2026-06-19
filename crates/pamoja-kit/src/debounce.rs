//! Debouncing a chattering on/off signal.

/// Cleans a noisy boolean signal by requiring it to hold steady before reporting a change.
///
/// A mechanical switch, a relay contact, or a reading crossing a threshold does not flip
/// cleanly: for a few milliseconds it chatters between states. A [`Debounce`] reports a
/// change only after the new value has been seen for a set number of consecutive samples,
/// so a button press, a float-switch trip, or a threshold crossing reads as one clean
/// event. This is the standard counter debounce: N stable samples accept a change, and any
/// contrary sample resets the count. At a fixed sample rate, N samples is the debounce time
/// - sampling every 5 ms with `samples` of `4` debounces over 20 ms.
///
/// # Examples
///
/// ```
/// use pamoja_kit::Debounce;
///
/// // A button needs three stable samples to register.
/// let mut button = Debounce::new(3, false);
/// assert!(!button.update(true)); // first press sample
/// assert!(!button.update(false)); // the contact bounced back
/// assert!(!button.update(true)); // counting restarts
/// assert!(!button.update(true));
/// assert!(button.update(true)); // three in a row: pressed
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Debounce {
    state: bool,
    candidate: bool,
    count: u16,
    samples: u16,
}

impl Debounce {
    /// Creates a debouncer.
    ///
    /// # Arguments
    ///
    /// * `samples` - consecutive stable samples required to accept a change. `0` and `1`
    ///   both accept a change on the first contrary sample.
    /// * `initial` - the starting debounced state.
    ///
    /// # Returns
    ///
    /// A debouncer reporting `initial` until a change is confirmed.
    pub fn new(samples: u16, initial: bool) -> Self {
        Self {
            state: initial,
            candidate: initial,
            count: 0,
            samples,
        }
    }

    /// Feeds a raw sample and returns the debounced state.
    ///
    /// # Arguments
    ///
    /// * `raw` - the latest raw signal value.
    ///
    /// # Returns
    ///
    /// The debounced state after this sample. It changes only once a contrary value has
    /// held for the required number of samples.
    pub fn update(&mut self, raw: bool) -> bool {
        if raw == self.state {
            self.count = 0;
            self.candidate = self.state;
        } else {
            if raw == self.candidate {
                self.count = self.count.saturating_add(1);
            } else {
                self.candidate = raw;
                self.count = 1;
            }
            if self.count >= self.samples {
                self.state = raw;
                self.candidate = raw;
                self.count = 0;
            }
        }
        self.state
    }

    /// Returns the current debounced state.
    pub fn state(&self) -> bool {
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn change_is_accepted_after_enough_stable_samples() {
        let mut debounce = Debounce::new(3, false);
        assert!(!debounce.update(true));
        assert!(!debounce.update(true));
        assert!(debounce.update(true)); // the third stable sample
        assert!(debounce.state());
    }

    #[test]
    fn chatter_resets_the_count() {
        let mut debounce = Debounce::new(3, false);
        debounce.update(true);
        debounce.update(true);
        assert!(!debounce.update(false)); // bounce back to the held state
        assert!(!debounce.update(true)); // counting restarts
        assert!(!debounce.update(true));
        assert!(debounce.update(true)); // now three clean in a row
    }

    #[test]
    fn one_sample_flips_immediately() {
        let mut debounce = Debounce::new(1, false);
        assert!(debounce.update(true));
    }

    #[test]
    fn a_single_contrary_sample_does_not_flip() {
        let mut debounce = Debounce::new(2, true);
        assert!(debounce.update(false)); // one low sample: still high
        assert!(!debounce.update(false)); // a second low sample: now low
    }
}
