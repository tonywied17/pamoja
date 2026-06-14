//! A typed publish/subscribe event bus used internally and exposed to app code.

use crate::error::Result;

/// A typed event bus carrying events of a single type.
///
/// The bus is the internal nervous system of a zero-edge application: sensors and
/// transports publish events, and application logic subscribes to them.
pub trait EventBus {
    /// The event type carried by this bus.
    type Event;

    /// Publish an event to all current subscribers.
    async fn publish(&self, event: Self::Event) -> Result<()>;

    /// Await the next event for this subscriber, or `None` once the bus closes.
    async fn next_event(&mut self) -> Result<Option<Self::Event>>;
}
