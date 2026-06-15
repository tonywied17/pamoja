//! Node.js bindings for the pamoja SDK, generated with napi-rs.
//!
//! This crate is the generated low-level surface (the contract tier): it exposes
//! the Rust core and capability crates to JavaScript and TypeScript one-to-one. A
//! hand-written, idiomatic facade wraps it for everyday use; see the package's
//! TypeScript entry point.

use napi_derive::napi;

/// Returns the version of the native pamoja module.
#[napi]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[cfg(feature = "mqtt")]
mod mqtt;
