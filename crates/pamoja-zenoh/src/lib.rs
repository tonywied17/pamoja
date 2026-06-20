#![cfg_attr(not(test), no_std)]

//! Zenoh key-expression logic for the pamoja SDK.
//!
//! Zenoh addresses data by key expressions: `/`-joined chunks with a small, exact wildcard
//! language. Before any session is opened, a node has to know whether a key is well-formed, what
//! its one canonical spelling is, and whether a subscription pattern matches a published key. That
//! is pure string logic with no I/O, and getting it wrong silently drops or misroutes messages, so
//! it lives here as checked logic anchored to the Zenoh specification, ahead of the live transport.
//!
//! See [`keyexpr`] for the rules and the operations:
//!
//! - validity: a key expression is `/`-joined non-empty chunks with no leading, trailing, or
//!   doubled `/`, where `*` and `**` are whole-chunk wildcards and `$*` is a sub-chunk wildcard.
//! - canonical form: two expressions that select the same keys share one spelling, so equality is
//!   a string comparison; [`canonize`](keyexpr::canonize) produces it.
//! - matching: [`matches`](keyexpr::matches) tests whether a concrete key is selected by a pattern,
//!   the routing question a subscriber asks of every publication.
//!
//! Pattern-against-pattern intersection and inclusion, and the live `Transport` over a Zenoh
//! session, arrive with the networked layer, where they are cross-checked against Zenoh's own
//! implementation.
//!
//! # Examples
//!
//! ```
//! use pamoja_zenoh::keyexpr::{canonize, matches};
//!
//! // A subscription with a single-chunk wildcard selects a matching publication.
//! assert!(matches("room275/*/temperature", "room275/device1/temperature"));
//! assert!(!matches("room275/*/temperature", "room275/temperature"));
//!
//! // `**/*` is valid but not canonical; its one canonical spelling puts the `*` first.
//! assert_eq!(canonize("robot/sensor/**/*").as_deref(), Some("robot/sensor/*/**"));
//! ```

extern crate alloc;

pub mod keyexpr;
