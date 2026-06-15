//! The error model shared by every pamoja crate.
//!
//! A single [`Error`] type keeps failure handling uniform across capabilities and
//! maps cleanly onto each language binding's native error idiom, such as
//! exceptions or rejected promises.

use core::fmt;

/// The error type returned by all fallible pamoja operations.
///
/// This enum is `#[non_exhaustive]`: new variants may be added in future releases
/// without a breaking change, so downstream `match` expressions must include a
/// wildcard arm.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// A transport-level failure while connecting, sending, or receiving.
    ///
    /// The payload is a human-readable description provided by the transport.
    Transport(String),

    /// A device or peripheral input/output operation failed.
    ///
    /// The payload is a human-readable description of the I/O fault.
    Io(String),

    /// A payload could not be encoded or decoded.
    ///
    /// The payload describes the encoding or decoding fault.
    Codec(String),

    /// The operation targeted a resource that is closed or disconnected.
    Closed,

    /// The requested capability is not compiled into this build.
    ///
    /// The payload names the missing capability, for example `"mqtt"`.
    Unsupported(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Transport(message) => write!(f, "transport error: {message}"),
            Self::Io(message) => write!(f, "io error: {message}"),
            Self::Codec(message) => write!(f, "codec error: {message}"),
            Self::Closed => f.write_str("resource is closed"),
            Self::Unsupported(capability) => {
                write!(f, "unsupported capability: {capability}")
            }
        }
    }
}

impl std::error::Error for Error {}

/// A specialized [`core::result::Result`] whose error type is fixed to [`Error`].
pub type Result<T> = core::result::Result<T, Error>;
