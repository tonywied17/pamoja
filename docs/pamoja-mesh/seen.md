# pamoja-mesh::seen

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

Duplicate suppression for flooded packets.

## struct `SeenCache`

A fixed-size memory of the most recently seen packets, so a node relays each one once.

In a flood every node rebroadcasts what it hears, so the same packet reaches a node
from several neighbours. Without a memory of what it has already handled, a node would
relay every copy and the flood would multiply without bound. This cache remembers the
last `N` packet keys (a [`dedup_key`](crate::Frame::dedup_key), the source and sequence
id) in a ring, evicting the oldest as new ones arrive, so the test for "have I seen
this?" stays cheap and needs no allocation. `N` sets how far back the memory reaches;
a small power of two such as 32 or 64 suits a local mesh.

**Examples**

```
use pamoja_mesh::SeenCache;

let mut seen: SeenCache<8> = SeenCache::new();
assert!(seen.record((0x42, 1)));  // first time: newly recorded
assert!(!seen.record((0x42, 1))); // again: a duplicate
assert!(seen.record((0x42, 2)));  // a different packet
```

### `SeenCache <N>::new`

Creates an empty cache.

**Returns**

A cache holding no keys.

```rust
const fn new() -> Self
```

### `SeenCache <N>::contains`

Reports whether a key is currently remembered.

**Arguments**

* `key` - the packet key to look for, from [`dedup_key`](crate::Frame::dedup_key).

**Returns**

`true` if the key is in the cache.

```rust
fn contains(&self, key:(u32, u16)) -> bool
```

### `SeenCache <N>::record`

Records a key, reporting whether it was new.

This is the flood test: record the key of a received packet, and act on the packet
only when this returns `true`. The oldest remembered key is evicted once the cache
is full.

**Arguments**

* `key` - the packet key to record, from [`dedup_key`](crate::Frame::dedup_key).

**Returns**

`true` if the key was not already remembered (the packet is new), `false` if it was
(the packet is a duplicate).

```rust
fn record(&mut self, key:(u32, u16)) -> bool
```

