//! A single entry in a signed, hash-chained audit log.

use alloc::vec::Vec;

use pamoja_core::{Error, Result};
use pamoja_security::Signature;
use sha2::{Digest, Sha256};

// The fixed header before each payload: the index, the previous hash, the signature.
const HEADER_LEN: usize = 8 + 32 + 64;

/// One entry in a tamper-evident audit log.
///
/// An entry binds a payload both to its position in the log and to the entry before
/// it: it carries the payload, the entry's index, the digest of the previous entry
/// (the chain link), and a signature over this entry's digest. Because each entry
/// commits to the one before, altering, reordering, inserting, or dropping any entry
/// breaks the chain and is caught on verification.
#[derive(Clone, Debug)]
pub struct Entry {
    index: u64,
    prev: [u8; 32],
    signature: Signature,
    payload: Vec<u8>,
}

impl Entry {
    pub(crate) fn new(index: u64, prev: [u8; 32], signature: Signature, payload: Vec<u8>) -> Self {
        Self {
            index,
            prev,
            signature,
            payload,
        }
    }

    /// Returns this entry's position in the log, counting from zero.
    ///
    /// # Returns
    ///
    /// The entry index.
    pub fn index(&self) -> u64 {
        self.index
    }

    /// Returns the digest of the previous entry that this entry chains to.
    ///
    /// # Returns
    ///
    /// The previous entry's digest, or all zeros for the first entry.
    pub fn previous(&self) -> [u8; 32] {
        self.prev
    }

    /// Returns the entry's payload, such as an encoded reading.
    ///
    /// # Returns
    ///
    /// The payload bytes.
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    /// Returns the signature over this entry's digest.
    ///
    /// # Returns
    ///
    /// The entry's [`Signature`].
    pub fn signature(&self) -> &Signature {
        &self.signature
    }

    /// Computes this entry's digest: the hash the signature covers and the next
    /// entry chains to.
    ///
    /// # Returns
    ///
    /// The 32-byte SHA-256 digest over the index, previous digest, and payload.
    pub fn digest(&self) -> [u8; 32] {
        digest(self.index, &self.prev, &self.payload)
    }

    /// Encodes the entry to bytes for durable storage.
    ///
    /// The layout is the little-endian index, the previous digest, the signature,
    /// then the payload.
    ///
    /// # Returns
    ///
    /// The encoded entry.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(HEADER_LEN + self.payload.len());
        bytes.extend_from_slice(&self.index.to_le_bytes());
        bytes.extend_from_slice(&self.prev);
        bytes.extend_from_slice(&self.signature.to_bytes());
        bytes.extend_from_slice(&self.payload);
        bytes
    }

    /// Decodes an entry from its stored bytes.
    ///
    /// # Arguments
    ///
    /// * `bytes` - the encoded entry, as produced by [`to_bytes`](Entry::to_bytes).
    ///
    /// # Returns
    ///
    /// The decoded entry.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Codec`](pamoja_core::Error::Codec) if `bytes` is shorter than
    /// an entry header.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < HEADER_LEN {
            return Err(Error::Codec("audit entry is shorter than its header".into()));
        }
        let index = u64::from_le_bytes(bytes[..8].try_into().expect("eight index bytes"));
        let prev: [u8; 32] = bytes[8..40].try_into().expect("thirty-two previous bytes");
        let signature: [u8; 64] = bytes[40..HEADER_LEN]
            .try_into()
            .expect("sixty-four signature bytes");
        let payload = bytes[HEADER_LEN..].to_vec();
        Ok(Self::new(
            index,
            prev,
            Signature::from_bytes(&signature),
            payload,
        ))
    }
}

// The digest the signature covers and the next entry chains to.
pub(crate) fn digest(index: u64, prev: &[u8; 32], payload: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(index.to_le_bytes());
    hasher.update(prev);
    hasher.update(payload);
    let out = hasher.finalize();
    let mut digest = [0u8; 32];
    digest.copy_from_slice(&out);
    digest
}

#[cfg(test)]
mod tests {
    use super::*;
    use pamoja_security::DeviceIdentity;

    #[test]
    fn an_entry_round_trips_through_bytes() {
        let device = DeviceIdentity::from_seed(&[1u8; 32]);
        let signature = device.sign(b"x");
        let entry = Entry::new(3, [7u8; 32], signature, b"payload".to_vec());

        let bytes = entry.to_bytes();
        let restored = Entry::from_bytes(&bytes).expect("parse");

        assert_eq!(restored.index(), 3);
        assert_eq!(restored.previous(), [7u8; 32]);
        assert_eq!(restored.payload(), b"payload");
        assert_eq!(restored.to_bytes(), bytes);
    }

    #[test]
    fn a_short_buffer_is_rejected() {
        let result = Entry::from_bytes(&[0u8; 10]);
        assert!(matches!(result, Err(Error::Codec(_))));
    }
}
