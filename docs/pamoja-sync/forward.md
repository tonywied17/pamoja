# pamoja-sync::forward

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

Forwarding buffered records onto a transport when a link appears.

## fn `drain_to`

Drains `store` onto `transport`, publishing each record to `topic`, oldest first.

Each record is sent before it is removed from the store, so a send failure
leaves that record and every record after it buffered in order to retry later:
nothing is lost and nothing is reordered. Delivery is at-least-once, since a
crash between a successful send and the record's removal redelivers it on the
next run.

This is the "forward" half of store-and-forward: buffer with a
[`Store`](pamoja_core::Store) while offline, then call this when a link
appears.

**Arguments**

* `store` - the queue to drain.
* `transport` - a connected transport to publish on.
* `topic` - the topic every record is published to.

**Returns**

The number of records forwarded once the store is drained empty.

**Errors**

Returns the transport's error if a send fails, leaving the unsent records
buffered in order, or [`Error::Io`](pamoja_core::Error::Io) if the store
cannot be read.

**Examples**

```no_run
use pamoja_core::{Store, Transport};
use pamoja_sync::{drain_to, MemoryStore};

let mut outbox = MemoryStore::new();
outbox.append(b"21.5").await?;
let forwarded = drain_to(&mut outbox, transport, "sensors/1/temperature").await?;
assert_eq!(forwarded, 1);
```

```rust
async fn drain_to <S, T>(store: &mut S, transport: &mut T, topic: &str) -> Result <usize> where S: Store, T: Transport,
```

