//! The zero-edge engine.
//!
//! This crate defines the abstractions that every capability crate (MQTT, serial,
//! CAN, ROS2, ...) implements. The core deliberately knows nothing about any
//! concrete protocol; it knows about [`Device`], [`Sensor`], [`Actuator`],
//! [`Transport`], [`Store`], and the [`EventBus`]. Concrete crates provide the
//! implementations, and applications depend only on the pieces they need.
//!
//! Phase 0 status: these are the trait skeletons. Method bodies arrive with the
//! first concrete capability crates in Phase 1. The core is structured to become
//! `no_std`-compatible (with `alloc`) so it can run on microcontrollers.

// Public traits use `async fn`, which is intentional for this static-dispatch SDK.
#![allow(async_fn_in_trait)]

pub mod bus;
pub mod device;
pub mod error;
pub mod store;
pub mod transport;

pub use bus::EventBus;
pub use device::{Actuator, Device, Sensor, Telemetry};
pub use error::{Error, Result};
pub use store::Store;
pub use transport::Transport;
