//! A UDP [`ByteLink`], the transport PX4 SITL uses and a common ground-station link.
//!
//! MAVLink over UDP is connectionless: a ground station binds a port and learns the vehicle's
//! address from the first datagram it receives, then replies to it. [`UdpLink`] supports that
//! bind-and-learn pattern with [`bind`](UdpLink::bind), and the send-first pattern (talking to a
//! vehicle at a known address, as with PX4's offboard port) with [`connect`](UdpLink::connect).
//! Each MAVLink frame is written as one datagram, which stays well under the MTU.

use std::io;
use std::net::SocketAddr;

use tokio::net::{ToSocketAddrs, UdpSocket};

use super::link_fault;
use crate::error::{MavlinkError, Result};
use crate::link::ByteLink;

/// A UDP socket presented as a [`ByteLink`].
///
/// The peer is either learned from the first datagram received (in [`bind`](UdpLink::bind)
/// mode) or set up front (in [`connect`](UdpLink::connect) mode), and every received datagram
/// refreshes it, so a vehicle that moves ports is followed.
pub struct UdpLink {
    socket: UdpSocket,
    peer: Option<SocketAddr>,
}

impl UdpLink {
    /// Binds a local address and learns the peer from the first datagram received.
    ///
    /// Read before writing in this mode: a write before any datagram has arrived has no peer to
    /// send to. This suits a vehicle configured to send its telemetry to this port.
    ///
    /// # Arguments
    ///
    /// * `local` - the local address to bind, such as `"0.0.0.0:14550"`.
    ///
    /// # Returns
    ///
    /// The bound link, with no peer yet.
    ///
    /// # Errors
    ///
    /// Returns an [`io::Error`] if the address cannot be bound.
    pub async fn bind(local: impl ToSocketAddrs) -> io::Result<Self> {
        let socket = UdpSocket::bind(local).await?;
        Ok(UdpLink { socket, peer: None })
    }

    /// Binds a local address and sets a fixed peer to send to.
    ///
    /// This suits talking to a vehicle at a known address, such as PX4's offboard UDP port.
    ///
    /// # Arguments
    ///
    /// * `local` - the local address to bind, such as `"0.0.0.0:0"`.
    /// * `remote` - the vehicle's address to send to.
    ///
    /// # Returns
    ///
    /// The link, ready to send.
    ///
    /// # Errors
    ///
    /// Returns an [`io::Error`] if the address cannot be bound.
    pub async fn connect(local: impl ToSocketAddrs, remote: SocketAddr) -> io::Result<Self> {
        let socket = UdpSocket::bind(local).await?;
        Ok(UdpLink {
            socket,
            peer: Some(remote),
        })
    }

    /// Returns the local address the socket is bound to.
    ///
    /// # Returns
    ///
    /// The bound local address.
    ///
    /// # Errors
    ///
    /// Returns an [`io::Error`] if the address cannot be read.
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.socket.local_addr()
    }

    /// Returns the peer address, once one is known.
    ///
    /// # Returns
    ///
    /// The peer address, or [`None`] before any datagram has been received in bind mode.
    pub fn peer(&self) -> Option<SocketAddr> {
        self.peer
    }
}

impl ByteLink for UdpLink {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let (n, from) = self.socket.recv_from(buf).await.map_err(link_fault)?;
        self.peer = Some(from);
        Ok(n)
    }

    async fn write_all(&mut self, data: &[u8]) -> Result<()> {
        let peer = self.peer.ok_or(MavlinkError::Closed)?;
        let sent = self.socket.send_to(data, peer).await.map_err(link_fault)?;
        if sent != data.len() {
            return Err(MavlinkError::Closed);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dialect::{Heartbeat, Message};
    use crate::link::Connection;

    #[tokio::test]
    async fn a_frame_crosses_a_real_udp_socket_pair() {
        // A vehicle end binds and a ground-station end sends to it; the frame crosses a real
        // localhost UDP socket, exercising the driver rather than an in-memory pipe.
        let vehicle = UdpLink::bind("127.0.0.1:0").await.unwrap();
        let vehicle_addr = vehicle.local_addr().unwrap();
        let gcs = UdpLink::connect("127.0.0.1:0", vehicle_addr).await.unwrap();

        let mut vehicle = Connection::new(vehicle, 1, 1);
        let mut gcs = Connection::new(gcs, 255, 190);

        let heartbeat = Heartbeat {
            custom_mode: 0,
            type_: 2,
            autopilot: 3,
            base_mode: 0,
            system_status: 4,
            mavlink_version: 3,
        };
        gcs.send(&heartbeat).await.unwrap();
        let frame = vehicle.recv().await.unwrap();
        assert_eq!(frame.message_id(), Heartbeat::ID);

        // The vehicle now knows the ground station's address and can reply.
        vehicle.send(&heartbeat).await.unwrap();
        let reply = gcs.recv().await.unwrap();
        assert_eq!(reply.message_id(), Heartbeat::ID);
    }
}
