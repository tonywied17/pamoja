//! An in-memory store-and-forward queue.

use std::collections::VecDeque;

use pamoja_core::{Error, Result, Store};

/// A fast in-memory first-in first-out queue.
///
/// Records live only for the lifetime of the process, which suits tests, the
/// simulators, and the upper tier of a layered store. An optional capacity caps
/// the number of buffered records and turns a full queue into an explicit
/// backpressure signal rather than letting memory grow without bound.
///
/// # Examples
///
/// ```no_run
/// use pamoja_core::Store;
/// use pamoja_sync::MemoryStore;
///
/// # async fn run() -> pamoja_core::Result<()> {
/// let mut store = MemoryStore::new();
/// store.append(b"first").await?;
/// store.append(b"second").await?;
/// assert_eq!(store.len().await?, 2);
/// assert_eq!(store.pop().await?, Some(b"first".to_vec()));
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug, Default)]
pub struct MemoryStore {
    records: VecDeque<Vec<u8>>,
    capacity: Option<usize>,
}

impl MemoryStore {
    /// Creates an unbounded in-memory store.
    ///
    /// # Returns
    ///
    /// An empty store that grows to hold as many records as memory allows.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a store that buffers at most `capacity` records.
    ///
    /// # Arguments
    ///
    /// * `capacity` - the maximum number of records to buffer; once reached,
    ///   [`append`](Store::append) reports backpressure instead of growing.
    ///
    /// # Returns
    ///
    /// An empty capacity-bounded store.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            records: VecDeque::new(),
            capacity: Some(capacity),
        }
    }
}

impl Store for MemoryStore {
    async fn append(&mut self, record: &[u8]) -> Result<()> {
        if let Some(capacity) = self.capacity {
            if self.records.len() >= capacity {
                return Err(Error::Io("store is at capacity".to_owned()));
            }
        }
        self.records.push_back(record.to_vec());
        Ok(())
    }

    async fn peek(&self) -> Result<Option<Vec<u8>>> {
        Ok(self.records.front().cloned())
    }

    async fn pop(&mut self) -> Result<Option<Vec<u8>>> {
        Ok(self.records.pop_front())
    }

    async fn len(&self) -> Result<usize> {
        Ok(self.records.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn drains_in_first_in_first_out_order() {
        let mut store = MemoryStore::new();
        store.append(b"a").await.expect("append");
        store.append(b"b").await.expect("append");
        store.append(b"c").await.expect("append");

        assert_eq!(store.len().await.expect("len"), 3);
        assert_eq!(store.pop().await.expect("pop"), Some(b"a".to_vec()));
        assert_eq!(store.pop().await.expect("pop"), Some(b"b".to_vec()));
        assert_eq!(store.pop().await.expect("pop"), Some(b"c".to_vec()));
        assert_eq!(store.pop().await.expect("pop"), None);
    }

    #[tokio::test]
    async fn peek_returns_the_oldest_without_removing_it() {
        let mut store = MemoryStore::new();
        assert_eq!(store.peek().await.expect("peek"), None);

        store.append(b"a").await.expect("append");
        store.append(b"b").await.expect("append");
        assert_eq!(store.peek().await.expect("peek"), Some(b"a".to_vec()));
        assert_eq!(store.len().await.expect("len"), 2);
        assert_eq!(store.pop().await.expect("pop"), Some(b"a".to_vec()));
    }

    #[tokio::test]
    async fn empty_store_reports_empty() {
        let store = MemoryStore::new();
        assert_eq!(store.len().await.expect("len"), 0);
        assert!(store.is_empty().await.expect("is_empty"));
    }

    #[tokio::test]
    async fn bounded_store_rejects_when_full() {
        let mut store = MemoryStore::with_capacity(1);
        store.append(b"first").await.expect("append");

        assert!(matches!(store.append(b"second").await, Err(Error::Io(_))));
        assert_eq!(store.len().await.expect("len"), 1);

        // Draining frees a slot so appends succeed again.
        assert_eq!(store.pop().await.expect("pop"), Some(b"first".to_vec()));
        store.append(b"second").await.expect("append after drain");
        assert_eq!(store.pop().await.expect("pop"), Some(b"second".to_vec()));
    }
}
