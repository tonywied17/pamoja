//! Device profiles: named, ready-to-run nodes assembled from pamoja capabilities.
//!
//! Most people who can put a sensor to good use are not electrical engineers, and
//! the gap between "I can read a sensor" and "I built something that works and warns
//! me when it fails" is wiring, tuning, and glue code. A device profile closes that
//! gap. It is a named, pre-wired bundle - a control policy, a publish topic, and a
//! power schedule - that a builder instantiates instead of choosing algorithms and
//! constants by hand.
//!
//! A profile has two halves:
//!
//! - [`Profile`] is the manifest: plain data a community can publish and share,
//!   carrying a [`ControlSpec`] and a [`PowerSchedule`]. It serializes to and from
//!   JSON with [`Profile::to_json`] and [`Profile::from_json`], so a profile can ship
//!   as a file and be loaded onto a device. The presets
//!   [`Profile::vaccine_fridge_monitor`], [`Profile::irrigation_node`],
//!   [`Profile::well_level`], and [`Profile::flood_sensor`] are convenience
//!   constructors for the same data.
//! - [`Node`] is what the runtime assembles from a profile and real components: a
//!   [`Sensor`](pamoja_core::Sensor), an [`Actuator`](pamoja_core::Actuator), a
//!   [`Transport`](pamoja_core::Transport), and a [`Codec`](pamoja_codec::Codec).
//!   Each [`tick`](Node::tick) reads, decides, drives the output, and publishes.
//!
//! The decision logic is a [`Controller`] that composes the `pamoja-kit` helpers, so
//! a profile is glue over field-tested math rather than new behavior. The node's I/O
//! is async; its decisions are synchronous and hardware-free, so the whole control
//! policy is unit-testable with no devices and no network.
//!
//! # Examples
//!
//! Assemble a cold-chain monitor's policy and evaluate a reading, no hardware needed:
//!
//! ```
//! use pamoja_profile::{Alert, Profile};
//!
//! let profile = Profile::vaccine_fridge_monitor();
//! let mut control = profile.controller();
//!
//! // A warm fridge: the cooler runs and a spoilage excursion is flagged.
//! let reaction = control.evaluate(9.0);
//! assert_eq!(reaction.actuator, Some(true));
//! assert!(matches!(reaction.alert, Some(Alert::OutOfRange { .. })));
//! ```

// The public traits this crate composes use `async fn`, matching the core SDK.
#![allow(async_fn_in_trait)]

mod control;
mod node;
mod profile;

pub use control::{Alert, Controller, Reaction};
pub use node::{NoActuator, Node};
pub use profile::{ControlSpec, PowerSchedule, Profile};
