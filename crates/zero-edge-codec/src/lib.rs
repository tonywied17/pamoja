//! Pluggable encode/decode for zero-edge payloads.
//!
//! Concrete wire formats (CBOR for constrained devices, Protobuf, JSON, or raw
//! framing) implement [`Codec`]. Hot paths are expected to favor zero-copy and
//! delta encoding so the SDK stays usable on metered, low-bandwidth links.
//!
//! Phase 0 status: the trait is defined; concrete formats land in Phase 1.

use zero_edge_core::Result;

/// Encodes and decodes values of type `T` to and from bytes.
pub trait Codec<T> {
    /// Encode `value` into a byte buffer.
    fn encode(&self, value: &T) -> Result<Vec<u8>>;

    /// Decode a value of type `T` from `bytes`.
    fn decode(&self, bytes: &[u8]) -> Result<T>;
}
