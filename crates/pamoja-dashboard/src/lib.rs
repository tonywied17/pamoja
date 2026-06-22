//! Local-first dashboard for a pamoja node.
//!
//! A node serves its own dashboard over its own WiFi hotspot, so a clinic worker, a
//! farmer, or a water committee sees their own data with no internet at all, in their
//! own language, on whatever cheap phone they have. This crate is the host side of
//! that: it turns the state a node already holds into a small, language-neutral
//! snapshot and serves a hand-built, localized page that renders it.
//!
//! The design rests on one split. The device emits only a [`State`] - stable keys,
//! stable codes, raw values, and canonical units, identical in every locale - and the
//! page does all rendering, formatting, and translation at the surface. That keeps the
//! device's job tiny enough for constrained hardware and the page's job rich enough to
//! be beautiful, and it means localization is a property of the page, not a fork of
//! the data.
//!
//! The pieces:
//!
//! - [`State`] is the language-neutral fleet snapshot served at `GET /state`:
//!   [`Org`]s of [`Group`]s of [`Sensor`]s, each group on its own [`Link`].
//! - [`StateSource`] is the one seam between the dashboard and its data; a real
//!   gateway and the [`Mock`] both implement it.
//! - [`Mock`] serves a deterministic [`Scenario`] so the whole dashboard runs and is
//!   debugged with no hardware.
//! - [`Server`] serves the page, the snapshot, and a live event stream over plain TCP.
//!
//! # Examples
//!
//! Render an alarm with no hardware and read it back as the JSON the page would fetch:
//!
//! ```
//! use pamoja_dashboard::{Mock, Scenario, StateSource, Status};
//!
//! let mut node = Mock::new(Scenario::Alarm);
//! let state = node.snapshot();
//! assert_eq!(state.status, Status::Alarm);
//!
//! let json = state.to_json().expect("serialize");
//! assert!(json.contains("\"status\":\"alarm\""));
//! ```

mod assets;
mod mock;
mod source;
mod state;

#[cfg(feature = "serve")]
mod serve;

pub use assets::Assets;
pub use mock::{Mock, Scenario};
pub use source::StateSource;
pub use state::{
    EventLevel, EventRecord, Group, Link, LinkKind, Mode, Org, Reading, Sensor, State, Status,
    Trend,
};

#[cfg(feature = "serve")]
pub use serve::Server;
