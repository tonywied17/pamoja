# pamoja-sync::file

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

A crash-safe on-disk store-and-forward queue.

## struct `FileStore`

A durable first-in first-out queue backed by one file per record.

Each [`append`](Store::append) writes a record to its own sequence-numbered
file, flushes it to disk, and atomically renames it into place, so a power
loss mid-write leaves the queue consistent: a partially written record is
never visible. [`pop`](Store::pop) reads and deletes the oldest record. The
directory itself is the durable state, so a store reopened after a crash
resumes with every record that was fully written.

Delivery is at-least-once: if the process stops between reading a record and
deleting it, the next [`open`](FileStore::open) returns that record again, so
consumers must tolerate the occasional redelivery.

**Examples**

```no_run
use pamoja_core::Store;
use pamoja_sync::FileStore;

let mut store = FileStore::open("/var/lib/pamoja/outbox")?;
store.append(b"reading").await?;
if let Some(record) = store.pop().await? {
    // forward `record` over a transport, then it is gone from the queue
    let _ = record;
}
```

### `FileStore::open`

Opens a store rooted at `dir`, creating the directory if needed.

Records left in the directory by a previous run are adopted in sequence
order, so the queue resumes where it left off.

**Arguments**

* `dir` - the directory that holds the queue's record files.

**Returns**

A store ready to append and drain records.

**Errors**

Returns [`Error::Io`](pamoja_core::Error::Io) if the directory cannot be
created or scanned.

```rust
fn open(dir: impl AsRef <Path>) -> Result <Self>
```

