//! Node.js bindings for the zero-edge core, generated with napi-rs.
//!
//! This module is intentionally thin: it exposes the Rust core to JavaScript and
//! TypeScript. Richer, idiomatic helpers belong in a hand-written layer on top of
//! the generated surface.

use napi_derive::napi;
use zero_edge_core::Error;

/// Returns the version string of the native zero-edge core module.
#[napi]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Formats a transport error message through the shared core error model.
///
/// This exists to prove the native core is linked and callable from Node: it
/// constructs a core error value from a JavaScript string and returns its
/// rendered form.
#[napi]
pub fn format_transport_error(message: String) -> String {
    Error::Transport(message).to_string()
}
