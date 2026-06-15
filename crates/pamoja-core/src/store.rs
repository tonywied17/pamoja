//! Durable local storage backing the offline-first synchronization layer.

use crate::error::Result;

/// A durable, first-in first-out queue for store-and-forward buffering.
///
/// Records are appended while a device is offline and drained in order when a
/// link becomes available, letting applications tolerate intermittent
/// connectivity without losing data.
pub trait Store {
    /// Appends a record to the back of the queue.
    ///
    /// # Arguments
    ///
    /// * `record` - the raw bytes to persist.
    ///
    /// # Returns
    ///
    /// `Ok(())` once the record is durably stored.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`](crate::Error::Io) if the record cannot be written to
    /// durable storage.
    async fn append(&mut self, record: &[u8]) -> Result<()>;

    /// Returns the oldest record without removing it.
    ///
    /// This lets a forwarder send a record before committing to its removal, so a
    /// failed send can leave the record buffered in order rather than dropping it.
    ///
    /// # Returns
    ///
    /// `Some(record)` containing the oldest buffered bytes, or `None` if the queue
    /// is empty.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`](crate::Error::Io) if the queue cannot be read.
    async fn peek(&self) -> Result<Option<Vec<u8>>>;

    /// Removes and returns the oldest record in the queue.
    ///
    /// # Returns
    ///
    /// `Some(record)` containing the oldest buffered bytes, or `None` if the queue
    /// is empty.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`](crate::Error::Io) if the queue cannot be read.
    async fn pop(&mut self) -> Result<Option<Vec<u8>>>;

    /// Returns the number of records currently buffered.
    ///
    /// # Returns
    ///
    /// The count of records waiting to be drained.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`](crate::Error::Io) if the queue length cannot be
    /// determined.
    async fn len(&self) -> Result<usize>;

    /// Returns whether the queue currently holds no records.
    ///
    /// The default implementation reports whether [`len`](Self::len) is zero.
    ///
    /// # Returns
    ///
    /// `true` if the queue is empty, `false` otherwise.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`](crate::Error::Io) if the queue length cannot be
    /// determined.
    async fn is_empty(&self) -> Result<bool> {
        Ok(self.len().await? == 0)
    }
}
