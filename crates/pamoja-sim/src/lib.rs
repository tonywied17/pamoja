//! Hardware-free device simulators for the pamoja SDK.
//!
//! The SDK is meant to be built and tested with no hardware at all, and that promise
//! only holds if there are convincing stand-ins for the parts a device would have.
//! This crate provides those stand-ins as ordinary implementations of the core
//! [`Sensor`](pamoja_core::Sensor) and [`Actuator`](pamoja_core::Actuator) traits, so
//! they drop into a `Node`, a profile, or a test exactly where a real driver will go
//! once the hardware-I/O layer lands:
//!
//! - [`SimSensor`] - a fake sensor that generates a lifelike signal from a baseline,
//!   a drift, and bounded, seedable noise, so a control loop meets the kind of messy
//!   input it will see in the field.
//! - [`Replay`] - a fake sensor that plays back an exact sequence of readings, for
//!   deterministic tests and scripted demos.
//! - [`RecordingActuator`] - a fake actuator that logs every command instead of
//!   driving hardware, so a test can assert what a control loop decided to do.
//! - [`DegradedLink`] - a [`Transport`](pamoja_core::Transport) decorator that
//!   simulates a lossy and intermittent radio link, so offline-first store-and-
//!   forward can be proven against a realistic bad network rather than assumed.
//! - [`SimRobot`] - a hardware-free differential-drive robot driven by a `Twist` and read back as
//!   a `Pose`, so a robot control loop can be developed and tested with no robot.
//!
//! # Examples
//!
//! Drive a recording relay from a scripted probe, with no hardware:
//!
//! ```
//! use pamoja_core::{Actuator, Sensor};
//! use pamoja_sim::{RecordingActuator, Replay};
//!
//! # async fn demo() -> pamoja_core::Result<()> {
//! let mut probe = Replay::new(vec![3.0, 7.0]);
//! let mut relay = RecordingActuator::new();
//! let log = relay.log();
//!
//! // Switch the relay on whenever the probe reads warm.
//! while let Ok(reading) = probe.read().await {
//!     relay.apply(reading > 5.0).await?;
//! }
//!
//! assert_eq!(log.commands(), vec![false, true]);
//! # Ok(())
//! # }
//! ```

mod actuator;
mod link;
mod robot;
mod sensor;

pub use actuator::{ActuatorLog, RecordingActuator};
pub use link::DegradedLink;
pub use robot::SimRobot;
pub use sensor::{Replay, SimSensor};
