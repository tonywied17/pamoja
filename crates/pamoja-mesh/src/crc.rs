//! CRC-16/CCITT-FALSE, the integrity check the mesh frame carries.

/// An incremental CRC-16/CCITT-FALSE accumulator.
///
/// CCITT-FALSE is the long-standing checksum of short radio frames: polynomial `0x1021`,
/// initial value `0xFFFF`, no reflection, and no final inversion. The accumulator lets a
/// checksum span more than one slice, which the mesh frame needs because it sums its
/// header and its payload while skipping the mutable hop-limit byte between them. For a
/// single contiguous slice, [`crc16`] is the one-shot form.
///
/// # Examples
///
/// ```
/// use pamoja_mesh::Crc16;
///
/// // Summing in two parts matches summing the whole in one go.
/// let mut crc = Crc16::new();
/// crc.update(b"1234");
/// crc.update(b"56789");
/// assert_eq!(crc.finish(), 0x29B1);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Crc16 {
    state: u16,
}

impl Crc16 {
    /// Creates an accumulator primed with the CCITT-FALSE initial value.
    ///
    /// # Returns
    ///
    /// A fresh accumulator, ready for [`update`](Crc16::update).
    pub const fn new() -> Self {
        Crc16 { state: 0xFFFF }
    }

    /// Folds a slice of bytes into the running checksum.
    ///
    /// # Arguments
    ///
    /// * `data` - the bytes to add to the checksum.
    pub fn update(&mut self, data: &[u8]) {
        for &byte in data {
            self.state ^= u16::from(byte) << 8;
            for _ in 0..8 {
                if self.state & 0x8000 != 0 {
                    self.state = (self.state << 1) ^ 0x1021;
                } else {
                    self.state <<= 1;
                }
            }
        }
    }

    /// Returns the checksum of everything folded in so far.
    ///
    /// # Returns
    ///
    /// The 16-bit CRC.
    pub fn finish(&self) -> u16 {
        self.state
    }
}

impl Default for Crc16 {
    fn default() -> Self {
        Self::new()
    }
}

/// Computes the CRC-16/CCITT-FALSE of a single byte slice.
///
/// # Arguments
///
/// * `data` - the bytes to check.
///
/// # Returns
///
/// The 16-bit CRC.
///
/// # Examples
///
/// ```
/// use pamoja_mesh::crc16;
///
/// // The standard CRC-16/CCITT-FALSE check value over the ASCII digits "123456789".
/// assert_eq!(crc16(b"123456789"), 0x29B1);
/// ```
pub fn crc16(data: &[u8]) -> u16 {
    let mut crc = Crc16::new();
    crc.update(data);
    crc.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_the_standard_check_value() {
        assert_eq!(crc16(b"123456789"), 0x29B1);
    }

    #[test]
    fn an_empty_slice_is_the_initial_value() {
        assert_eq!(crc16(&[]), 0xFFFF);
    }

    #[test]
    fn updating_in_parts_matches_one_shot() {
        let mut split = Crc16::new();
        split.update(b"123");
        split.update(b"456789");
        assert_eq!(split.finish(), crc16(b"123456789"));
    }
}
