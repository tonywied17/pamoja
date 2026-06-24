# pamoja-sync::memory

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

An in-memory store-and-forward queue.

## struct `MemoryStore`

A fast in-memory first-in first-out queue.

Records live only for the lifetime of the process, which suits tests, the
simulators, and the upper tier of a layered store. An optional capacity caps
the number of buffered records and turns a full queue into an explicit
backpressure signal rather than letting memory grow without bound.

**Examples**

```no_run
use pamoja_core::Store;
use pamoja_sync::MemoryStore;

let mut store = MemoryStore::new();
store.append(b"first").await?;
store.append(b"second").await?;
assert_eq!(store.len().await?, 2);
assert_eq!(store.pop().await?, Some(b"first".to_vec()));
```

### `MemoryStore::new`

Creates an unbounded in-memory store.

**Returns**

An empty store that grows to hold as many records as memory allows.

```rust
fn new() -> Self
```

### `MemoryStore::with_capacity`

Creates a store that buffers at most `capacity` records.

**Arguments**

* `capacity` - the maximum number of records to buffer; once reached,
  [`append`](Store::append) reports backpressure instead of growing.

**Returns**

An empty capacity-bounded store.

```rust
fn with_capacity(capacity: usize) -> Self
```

