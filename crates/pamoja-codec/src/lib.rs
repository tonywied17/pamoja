//! Pluggable serialization for pamoja payloads.
//!
//! Concrete wire formats - CBOR for constrained devices, Protocol Buffers, JSON,
//! or raw framing - implement the [`Codec`] trait. This crate defines the trait
//! and provides serde-based implementations behind feature flags:
//!
//! - [`CborCodec`] (feature `cbor`, on by default) - compact binary framing for
//!   constrained devices and metered links.
//! - [`JsonCodec`] (feature `json`, on by default) - human-readable framing for
//!   interop and debugging.
//! - [`BytesCodec`] (always available) - a no-op codec that carries raw bytes.
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

mod bytes;
pub use bytes::BytesCodec;

#[cfg(feature = "cbor")]
mod cbor;
#[cfg(feature = "cbor")]
pub use cbor::CborCodec;

#[cfg(feature = "json")]
mod json;
#[cfg(feature = "json")]
pub use json::JsonCodec;

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
