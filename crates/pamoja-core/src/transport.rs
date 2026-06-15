//! The transport abstraction: how bytes move between the SDK and a device or peer.

use crate::error::Result;

/// A bidirectional, topic-addressed message transport.
///
/// Implementations include MQTT, CoAP, LoRa, serial, and CAN. They are expected
/// to handle reconnection and backpressure internally so that callers see a
/// uniform, protocol-agnostic surface.
pub trait Transport {
    /// Establishes the connection to the broker, peer, or bus.
    ///
    /// # Returns
    ///
    /// `Ok(())` once the transport is connected and ready to carry traffic.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Transport`](crate::Error::Transport) if the connection
    /// cannot be established.
    async fn connect(&mut self) -> Result<()>;

    /// Publishes a payload to a topic.
    ///
    /// # Arguments
    ///
    /// * `topic` - the destination topic or channel address.
    /// * `payload` - the raw bytes to publish.
    ///
    /// # Returns
    ///
    /// `Ok(())` once the payload has been handed to the transport for delivery.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Transport`](crate::Error::Transport) if the payload cannot
    /// be sent, or [`Error::Closed`](crate::Error::Closed) if the transport is not
    /// connected.
    async fn send(&mut self, topic: &str, payload: &[u8]) -> Result<()>;

    /// Subscribes to a topic so that matching payloads are routed to this transport.
    ///
    /// # Arguments
    ///
    /// * `topic` - the topic or channel filter to subscribe to.
    ///
    /// # Returns
    ///
    /// `Ok(())` once the subscription is registered with the transport.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Transport`](crate::Error::Transport) if the subscription
    /// is rejected, or [`Error::Closed`](crate::Error::Closed) if the transport is
    /// not connected.
    async fn subscribe(&mut self, topic: &str) -> Result<()>;
}
