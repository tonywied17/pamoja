#![cfg_attr(not(test), no_std)]

//! Goal-named helper math for the pamoja SDK.
//!
//! The hardest part of building something real on a cheap sensor is rarely reading
//! the sensor; it is turning that reading into a decision that holds up in the
//! field. `pamoja-kit` is the layer that closes that gap for people who are not
//! signal-processing engineers. Each helper is named for the goal rather than the
//! technique, ships with correct defaults, and documents the real algorithm one
//! layer down, so it teaches as it abstracts.
//!
//! The first helpers cover the jobs the cookbook leans on most:
//!
//! - [`Smoother`] - smooth a noisy reading (exponential moving average).
//! - [`Median`] - reject spikes with a rolling median (robust to a lone bad reading).
//! - [`Debounce`] - clean a chattering on/off signal into one event (counter debounce).
//! - [`Calibration`] - turn a raw reading into real units (two-point linear map).
//! - [`Thermostat`] - keep a reading near a setpoint (on/off control with
//!   hysteresis).
//! - [`Depletion`] - warn before a falling level runs out (linear extrapolation).
//! - [`Surge`] - warn when a reading changes dangerously fast (first difference).
//! - [`Trend`] - tell whether a value is rising or falling and how fast (least-squares slope).
//! - [`Window`] - keep a rolling window of recent readings and read their spread
//!   (min, max, range, mean, population variance).
//! - [`units`] - convert a reading to the unit a person reads (Celsius and Fahrenheit,
//!   pascals to hPa/kPa/psi, ratio and percent).
//! - [`Geofence`] - warn when a tracked point leaves a safe area (great-circle
//!   distance). Behind the default `geo` feature, which pulls in `libm` for the
//!   trigonometry; disable it to keep the crate dependency-free.
//!
//! The crate is `no_std` and allocation-free, so the same helpers run on a
//! microcontroller and on a server.
//!
//! # Examples
//!
//! Compose two helpers to hold a noisy fridge probe near 4 C:
//!
//! ```
//! use pamoja_kit::{Smoother, Thermostat};
//!
//! let mut probe = Smoother::new(0.5);
//! let mut fridge = Thermostat::cooling(4.0, 0.5);
//!
//! // A warm, noisy reading is smoothed, then drives the cooler on.
//! let cooler_on = fridge.update(probe.update(7.8));
//! assert!(cooler_on);
//! ```

mod calibration;
mod debounce;
mod depletion;
mod median;
mod smoothing;
mod surge;
mod thermostat;
mod trend;
mod window;

pub mod units;

#[cfg(feature = "geo")]
mod geo;

pub use calibration::Calibration;
pub use debounce::Debounce;
pub use depletion::Depletion;
pub use median::Median;
pub use smoothing::Smoother;
pub use surge::Surge;
pub use thermostat::Thermostat;
pub use trend::Trend;
pub use window::Window;

#[cfg(feature = "geo")]
pub use geo::{Boundary, Coordinate, Geofence};
