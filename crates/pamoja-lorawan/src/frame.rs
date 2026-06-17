//! The LoRaWAN PHYPayload and the small shared pieces of its header.

use crate::error::LorawanError;

/// The largest PHYPayload, in bytes, this crate builds or accepts.
///
/// Comfortably above the largest regional LoRaWAN maximum, so a frame always fits.
pub const MAX_FRAME: usize = 256;

/// The largest application payload, in bytes, a single frame can carry (with no frame
/// options present).
pub const MAX_PAYLOAD: usize = MAX_FRAME - 13;

// MType values, in the top three bits of the MHDR.
pub(crate) const MTYPE_UNCONFIRMED_UP: u8 = 0x40;
pub(crate) const MTYPE_UNCONFIRMED_DOWN: u8 = 0x60;
pub(crate) const MTYPE_CONFIRMED_UP: u8 = 0x80;
pub(crate) const MTYPE_CONFIRMED_DOWN: u8 = 0xA0;
// The mask selecting the MType bits of the MHDR.
pub(crate) const MTYPE_MASK: u8 = 0xE0;

/// The direction a frame travels, which the MIC and the payload encryption both fold in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    /// From an end device up to the network.
    Uplink,
    /// From the network down to an end device.
    Downlink,
}

impl Direction {
    // The direction bit used in the MIC and encryption blocks: 0 up, 1 down.
    pub(crate) fn bit(self) -> u8 {
        match self {
            Direction::Uplink => 0,
            Direction::Downlink => 1,
        }
    }
}

/// An encoded LoRaWAN frame, the bytes that go on the air.
///
/// Built by a [`Session`](crate::Session) or a join exchange, and held in a fixed buffer
/// so encoding never allocates. [`as_bytes`](PhyPayload::as_bytes) hands the radio exactly
/// what to transmit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PhyPayload {
    bytes: [u8; MAX_FRAME],
    len: usize,
}

impl PhyPayload {
    // Copies an assembled frame into a fixed buffer.
    pub(crate) fn new(bytes: &[u8]) -> Result<Self, LorawanError> {
        if bytes.len() > MAX_FRAME {
            return Err(LorawanError::PayloadTooLong);
        }
        let mut buf = [0u8; MAX_FRAME];
        buf[..bytes.len()].copy_from_slice(bytes);
        Ok(PhyPayload {
            bytes: buf,
            len: bytes.len(),
        })
    }

    /// Returns the frame as bytes, ready to transmit.
    ///
    /// # Returns
    ///
    /// The whole PHYPayload, MIC included.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[..self.len]
    }
}
