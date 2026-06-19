//! SPI clock modes and bit order.
//!
//! SPI has no addressing and no framing of its own: a transfer is just bytes clocked in
//! and out at the same time. What a controller and a peripheral must agree on is when the
//! clock idles and which clock edge samples data - the two bits CPOL (clock polarity) and
//! CPHA (clock phase) - and whether each byte travels most- or least-significant bit
//! first. Datasheets quote the CPOL/CPHA pair as a single mode number from 0 to 3, and the
//! commonest cause of a dead SPI link is a transposed pair or the wrong mode. This module
//! makes the mode a checked value rather than two loose booleans a caller can swap.

/// An SPI clock mode: the `(CPOL, CPHA)` pair a controller and peripheral must share.
///
/// The mode number is `(CPOL << 1) | CPHA`, so the four modes are:
///
/// | Mode | CPOL | CPHA | Clock idles | Data sampled on |
/// | --- | --- | --- | --- | --- |
/// | 0 | 0 | 0 | low | leading edge (rising) |
/// | 1 | 0 | 1 | low | trailing edge (falling) |
/// | 2 | 1 | 0 | high | leading edge (falling) |
/// | 3 | 1 | 1 | high | trailing edge (rising) |
///
/// # Examples
///
/// ```
/// use pamoja_gpio::spi::Mode;
///
/// // An SD card and most LoRa radios use mode 0.
/// assert_eq!(Mode::Mode0.number(), 0);
/// assert_eq!(Mode::Mode0.cpol_cpha(), (false, false));
/// assert_eq!(Mode::from_number(3), Some(Mode::Mode3));
/// assert_eq!(Mode::from_cpol_cpha(true, false), Mode::Mode2);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// CPOL 0, CPHA 0: clock idles low, data sampled on the rising (leading) edge.
    Mode0,
    /// CPOL 0, CPHA 1: clock idles low, data sampled on the falling (trailing) edge.
    Mode1,
    /// CPOL 1, CPHA 0: clock idles high, data sampled on the falling (leading) edge.
    Mode2,
    /// CPOL 1, CPHA 1: clock idles high, data sampled on the rising (trailing) edge.
    Mode3,
}

impl Mode {
    /// Returns the mode number, `0..=3`, as datasheets quote it.
    ///
    /// # Returns
    ///
    /// The number `(CPOL << 1) | CPHA`.
    pub fn number(self) -> u8 {
        match self {
            Mode::Mode0 => 0,
            Mode::Mode1 => 1,
            Mode::Mode2 => 2,
            Mode::Mode3 => 3,
        }
    }

    /// Returns the mode a number names, if it is in range.
    ///
    /// # Arguments
    ///
    /// * `number` - a mode number.
    ///
    /// # Returns
    ///
    /// The matching [`Mode`], or [`None`] if `number` is above `3`.
    pub fn from_number(number: u8) -> Option<Mode> {
        match number {
            0 => Some(Mode::Mode0),
            1 => Some(Mode::Mode1),
            2 => Some(Mode::Mode2),
            3 => Some(Mode::Mode3),
            _ => None,
        }
    }

    /// Returns the `(CPOL, CPHA)` pair for this mode.
    ///
    /// # Returns
    ///
    /// `(clock idles high, data sampled on the trailing edge)`.
    pub fn cpol_cpha(self) -> (bool, bool) {
        match self {
            Mode::Mode0 => (false, false),
            Mode::Mode1 => (false, true),
            Mode::Mode2 => (true, false),
            Mode::Mode3 => (true, true),
        }
    }

    /// Returns the mode for a `(CPOL, CPHA)` pair.
    ///
    /// # Arguments
    ///
    /// * `cpol` - clock polarity: `true` if the clock idles high.
    /// * `cpha` - clock phase: `true` if data is sampled on the trailing edge.
    ///
    /// # Returns
    ///
    /// The matching [`Mode`]. Every pair maps to a mode, so this never fails.
    pub fn from_cpol_cpha(cpol: bool, cpha: bool) -> Mode {
        match (cpol, cpha) {
            (false, false) => Mode::Mode0,
            (false, true) => Mode::Mode1,
            (true, false) => Mode::Mode2,
            (true, true) => Mode::Mode3,
        }
    }

    /// Returns `true` if the clock idles high (CPOL = 1), which is modes 2 and 3.
    pub fn clock_idles_high(self) -> bool {
        self.cpol_cpha().0
    }

    /// Returns `true` if data is sampled on the trailing clock edge (CPHA = 1), which is
    /// modes 1 and 3.
    pub fn samples_on_trailing_edge(self) -> bool {
        self.cpol_cpha().1
    }
}

/// The order bits travel within each SPI byte.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BitOrder {
    /// Most-significant bit first. The common default for nearly every SPI peripheral.
    MsbFirst,
    /// Least-significant bit first.
    LsbFirst,
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [Mode; 4] = [Mode::Mode0, Mode::Mode1, Mode::Mode2, Mode::Mode3];

    #[test]
    fn number_round_trips() {
        for mode in ALL {
            assert_eq!(Mode::from_number(mode.number()), Some(mode));
        }
        assert_eq!(Mode::from_number(4), None);
    }

    #[test]
    fn number_is_cpol_shifted_over_cpha() {
        // The defining relation: mode = (CPOL << 1) | CPHA.
        for mode in ALL {
            let (cpol, cpha) = mode.cpol_cpha();
            assert_eq!(mode.number(), (u8::from(cpol) << 1) | u8::from(cpha));
        }
    }

    #[test]
    fn cpol_cpha_round_trips() {
        for mode in ALL {
            let (cpol, cpha) = mode.cpol_cpha();
            assert_eq!(Mode::from_cpol_cpha(cpol, cpha), mode);
        }
    }

    #[test]
    fn the_named_pairs_are_correct() {
        assert_eq!(Mode::Mode0.cpol_cpha(), (false, false));
        assert_eq!(Mode::Mode1.cpol_cpha(), (false, true));
        assert_eq!(Mode::Mode2.cpol_cpha(), (true, false));
        assert_eq!(Mode::Mode3.cpol_cpha(), (true, true));
        assert!(!Mode::Mode0.clock_idles_high() && !Mode::Mode0.samples_on_trailing_edge());
        assert!(Mode::Mode3.clock_idles_high() && Mode::Mode3.samples_on_trailing_edge());
    }
}
