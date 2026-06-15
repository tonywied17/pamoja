//! A typed publish/subscribe event bus used internally and by application code.

use crate::error::Result;

/// A typed publish/subscribe channel carrying events of a single type.
///
/// Producers such as sensors and transports publish events, and consumers await
/// them. The event type is fixed per bus so that delivery is statically typed.
pub trait EventBus {
    /// The event type carried by this bus.
    type Event;

    /// Publishes an event to every current subscriber.
    ///
    /// # Arguments
    ///
    /// * `event` - the event to broadcast, consumed by the call.
    ///
    /// # Returns
    ///
    /// `Ok(())` once the event has been accepted for delivery.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Closed`](crate::Error::Closed) if the bus has been shut
    /// down.
    async fn publish(&self, event: Self::Event) -> Result<()>;

    /// Awaits the next event for this subscriber.
    ///
    /// # Returns
    ///
    /// `Some(event)` when an event is available, or `None` once the bus is closed.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Closed`](crate::Error::Closed) if the bus has been shut
    /// down unexpectedly.
    async fn next_event(&mut self) -> Result<Option<Self::Event>>;
}
