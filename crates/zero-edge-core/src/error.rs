//! The error model shared across every zero-edge crate.
//!
//! A single [`Error`] type keeps behavior consistent across capabilities and maps
//! cleanly into each language binding's idioms (exceptions, rejected promises, and
//! so on).

use core::fmt;

/// The unified error type for the zero-edge SDK.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// A transport-level failure during connect, send, or receive.
    Transport(String),
    /// A device or peripheral I/O failure.
    Io(String),
    /// Encoding or decoding of a payload failed.
    Codec(String),
    /// An operation was attempted on a closed or disconnected resource.
    Closed,
    /// A capability was requested that is not compiled into this build.
    Unsupported(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Transport(msg) => write!(f, "transport error: {msg}"),
            Error::Io(msg) => write!(f, "io error: {msg}"),
            Error::Codec(msg) => write!(f, "codec error: {msg}"),
            Error::Closed => write!(f, "resource is closed"),
            Error::Unsupported(cap) => write!(f, "unsupported capability: {cap}"),
        }
    }
}

impl std::error::Error for Error {}

/// The result type used throughout the SDK.
pub type Result<T> = core::result::Result<T, Error>;
