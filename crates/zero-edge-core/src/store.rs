//! Durable local storage used by the offline-first sync layer.

use crate::error::Result;

/// A durable append/drain queue for store-and-forward buffering.
///
/// This is the backbone of offline-first operation: records are appended while a
/// device is disconnected and drained opportunistically when a link appears.
pub trait Store {
    /// Append a record to the durable queue.
    async fn append(&mut self, record: &[u8]) -> Result<()>;

    /// Remove and return the oldest record, or `None` if the queue is empty.
    async fn pop(&mut self) -> Result<Option<Vec<u8>>>;

    /// The number of records currently buffered.
    async fn len(&self) -> Result<usize>;

    /// Whether the queue currently holds no records.
    async fn is_empty(&self) -> Result<bool> {
        Ok(self.len().await? == 0)
    }
}
