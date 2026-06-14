//! The transport abstraction: how bytes move to and from a device or peer.

use crate::error::Result;

/// A bidirectional message transport such as MQTT, CoAP, serial, or CAN.
///
/// Implementations are expected to handle reconnect and backpressure internally
/// so that callers see a stable, capability-agnostic surface.
pub trait Transport {
    /// Establish the connection.
    async fn connect(&mut self) -> Result<()>;

    /// Publish `payload` to `topic`.
    async fn send(&mut self, topic: &str, payload: &[u8]) -> Result<()>;

    /// Subscribe to `topic` so that matching payloads are delivered to this transport.
    async fn subscribe(&mut self, topic: &str) -> Result<()>;
}
