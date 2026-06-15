//! Forwarding buffered records onto a transport when a link appears.

use pamoja_core::{Result, Store, Transport};

/// Drains `store` onto `transport`, publishing each record to `topic`, oldest first.
///
/// Each record is sent before it is removed from the store, so a send failure
/// leaves that record and every record after it buffered in order to retry later:
/// nothing is lost and nothing is reordered. Delivery is at-least-once, since a
/// crash between a successful send and the record's removal redelivers it on the
/// next run.
///
/// This is the "forward" half of store-and-forward: buffer with a
/// [`Store`](pamoja_core::Store) while offline, then call this when a link
/// appears.
///
/// # Arguments
///
/// * `store` - the queue to drain.
/// * `transport` - a connected transport to publish on.
/// * `topic` - the topic every record is published to.
///
/// # Returns
///
/// The number of records forwarded once the store is drained empty.
///
/// # Errors
///
/// Returns the transport's error if a send fails, leaving the unsent records
/// buffered in order, or [`Error::Io`](pamoja_core::Error::Io) if the store
/// cannot be read.
///
/// # Examples
///
/// ```no_run
/// use pamoja_core::{Store, Transport};
/// use pamoja_sync::{drain_to, MemoryStore};
///
/// # async fn run(transport: &mut impl Transport) -> pamoja_core::Result<()> {
/// let mut outbox = MemoryStore::new();
/// outbox.append(b"21.5").await?;
/// let forwarded = drain_to(&mut outbox, transport, "sensors/1/temperature").await?;
/// assert_eq!(forwarded, 1);
/// # Ok(())
/// # }
/// ```
pub async fn drain_to<S, T>(store: &mut S, transport: &mut T, topic: &str) -> Result<usize>
where
    S: Store,
    T: Transport,
{
    let mut forwarded = 0;
    while let Some(record) = store.peek().await? {
        transport.send(topic, &record).await?;
        store.pop().await?;
        forwarded += 1;
    }
    Ok(forwarded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MemoryStore;
    use pamoja_core::Error;

    /// A transport that records what it sends and can fail after a set count.
    #[derive(Default)]
    struct MockTransport {
        sent: Vec<(String, Vec<u8>)>,
        fail_after: Option<usize>,
    }

    impl Transport for MockTransport {
        async fn connect(&mut self) -> Result<()> {
            Ok(())
        }

        async fn send(&mut self, topic: &str, payload: &[u8]) -> Result<()> {
            if self.fail_after.is_some_and(|limit| self.sent.len() >= limit) {
                return Err(Error::Transport("link down".to_owned()));
            }
            self.sent.push((topic.to_owned(), payload.to_vec()));
            Ok(())
        }

        async fn subscribe(&mut self, _topic: &str) -> Result<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn drains_every_record_in_order() {
        let mut store = MemoryStore::new();
        for record in [b"a", b"b", b"c"] {
            store.append(record).await.expect("append");
        }
        let mut transport = MockTransport::default();

        let forwarded = drain_to(&mut store, &mut transport, "out")
            .await
            .expect("drain");

        assert_eq!(forwarded, 3);
        assert!(store.is_empty().await.expect("is_empty"));
        assert_eq!(
            transport.sent,
            vec![
                ("out".to_owned(), b"a".to_vec()),
                ("out".to_owned(), b"b".to_vec()),
                ("out".to_owned(), b"c".to_vec()),
            ]
        );
    }

    #[tokio::test]
    async fn send_failure_preserves_remaining_records_in_order() {
        let mut store = MemoryStore::new();
        for record in [b"a", b"b", b"c"] {
            store.append(record).await.expect("append");
        }
        let mut transport = MockTransport {
            fail_after: Some(1),
            ..Default::default()
        };

        let result = drain_to(&mut store, &mut transport, "out").await;

        assert!(matches!(result, Err(Error::Transport(_))));
        // Exactly the first record was forwarded and removed; the rest stay queued
        // in their original order for a later retry.
        assert_eq!(transport.sent, vec![("out".to_owned(), b"a".to_vec())]);
        assert_eq!(store.len().await.expect("len"), 2);
        assert_eq!(store.pop().await.expect("pop"), Some(b"b".to_vec()));
        assert_eq!(store.pop().await.expect("pop"), Some(b"c".to_vec()));
    }
}
