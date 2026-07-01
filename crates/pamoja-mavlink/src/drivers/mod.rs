//! Real [`ByteLink`](crate::link::ByteLink) drivers over serial ports, UDP, and TCP.
//!
//! The wire core and the [`Vehicle`](crate::vehicle::Vehicle) are written against the
//! [`ByteLink`](crate::link::ByteLink) seam, so a real autopilot is reached by plugging one of
//! these drivers into it: a serial line to a flight controller, or a UDP or TCP socket to a
//! SITL simulator or a ground-station bridge. PX4 SITL exposes MAVLink over UDP; ArduPilot SITL
//! exposes it over TCP (port 5760) and can also be told to send UDP; a companion computer wired
//! to a Pixhawk speaks it over serial.
//!
//! These drivers are available with the default `std` feature and run on the tokio runtime; the
//! serial driver additionally needs the `serial` feature, which pulls in a serial-port backend.

mod tcp;
mod udp;

pub use tcp::TcpLink;
pub use udp::UdpLink;

#[cfg(feature = "serial")]
mod serial;

#[cfg(feature = "serial")]
pub use serial::SerialLink;

// Maps a link IO error onto the wire layer's error model. A read or write that fails ends the
// link, which the connection treats the same as a clean end of input.
#[inline]
fn link_fault(_err: std::io::Error) -> crate::MavlinkError {
    crate::MavlinkError::Closed
}
