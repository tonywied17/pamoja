//! Pluggable serialization for pamoja payloads.
//!
//! Concrete wire formats - CBOR for constrained devices, Protocol Buffers, JSON,
//! or raw framing - implement the [`Codec`] trait. This crate defines the trait;
//! the format implementations are provided by the capability crates.
//!
//! # Examples
//!
//! A little-endian codec for `u32` values:
//!
//! ```
//! use pamoja_codec::Codec;
//! use pamoja_core::{Error, Result};
//!
//! struct LeU32;
//!
//! impl Codec<u32> for LeU32 {
//!     fn encode(&self, value: &u32) -> Result<Vec<u8>> {
//!         Ok(value.to_le_bytes().to_vec())
//!     }
//!
//!     fn decode(&self, bytes: &[u8]) -> Result<u32> {
//!         let array = bytes
//!             .try_into()
//!             .map_err(|_| Error::Codec("expected 4 bytes".into()))?;
//!         Ok(u32::from_le_bytes(array))
//!     }
//! }
//!
//! let codec = LeU32;
//! let encoded = codec.encode(&42).unwrap();
//! assert_eq!(codec.decode(&encoded).unwrap(), 42);
//! ```

use pamoja_core::Result;

/// Encodes and decodes values of type `T` to and from byte buffers.
///
/// A codec is the bridge between in-memory values and the bytes carried by a
/// [`Transport`](pamoja_core::Transport) or persisted by a
/// [`Store`](pamoja_core::Store).
pub trait Codec<T> {
    /// Encodes a value into a byte buffer.
    ///
    /// # Arguments
    ///
    /// * `value` - the value to serialize.
    ///
    /// # Returns
    ///
    /// A byte buffer containing the encoded representation of `value`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Codec`](pamoja_core::Error::Codec) if the value cannot
    /// be encoded.
    fn encode(&self, value: &T) -> Result<Vec<u8>>;

    /// Decodes a value from a byte buffer.
    ///
    /// # Arguments
    ///
    /// * `bytes` - the encoded representation to deserialize.
    ///
    /// # Returns
    ///
    /// The value decoded from `bytes`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Codec`](pamoja_core::Error::Codec) if `bytes` is not a
    /// valid encoding of `T`.
    fn decode(&self, bytes: &[u8]) -> Result<T>;
}
