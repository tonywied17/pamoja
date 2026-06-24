# pamoja-loopback::faulty

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

A transport decorator that injects send failures for degraded-link testing.

## struct `Faulty`

Wraps a [`Transport`] and fails a configurable number of upcoming sends.

This simulates an intermittent link so offline-first behavior can be proven
rather than assumed: pair it with a store-and-forward drain and assert that
every record still arrives, in order, once the link recovers.

**Examples**

```
use pamoja_core::Transport;
use pamoja_loopback::{Faulty, LoopbackBroker, LoopbackTransport};

let broker = LoopbackBroker::new();
let mut node = Faulty::new(LoopbackTransport::new(broker), 1);
node.connect().await?;

// The first send fails, simulating a dropped link; the next succeeds.
assert!(node.send("t", b"x").await.is_err());
node.send("t", b"x").await?;
```

### `Faulty <T>::new`

Wraps `inner`, failing its next `failures` sends before passing through.

**Arguments**

* `inner` - the transport to decorate.
* `failures` - how many of the next [`send`](Transport::send) calls fail.

**Returns**

A decorator that injects the requested failures, then delegates to `inner`.

```rust
fn new(inner: T, failures: usize) -> Self
```

### `Faulty <T>::fail_next`

Arms the decorator to fail the next `count` sends.

**Arguments**

* `count` - the number of upcoming sends to fail, simulating another link
  outage.

```rust
fn fail_next(&mut self, count: usize)
```

