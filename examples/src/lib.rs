//! Shared end-to-end scenarios that compose the pamoja crates.
//!
//! The runnable examples each exercise one slice of the SDK. This library holds the
//! scenarios that compose many crates at once, so a single run proves the pieces fit
//! together. A scenario is plain logic that returns its observable results, which lets
//! both a runnable example (which narrates them) and a conformance test (which asserts
//! on them) drive the exact same run.

pub mod cold_chain;
pub mod degraded_link;
