# pamoja-loopback::broker

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

The shared in-memory broker that routes loopback messages.

## struct `LoopbackBroker`

A shared, in-process router for [`LoopbackTransport`](crate::LoopbackTransport)s.

Clone a single broker into every transport that should share a namespace; a
publish on one transport is delivered to every transport whose subscriptions
match the topic. The broker is cheap to clone, and all clones share one
routing table.

### `LoopbackBroker::new`

Creates an empty broker.

**Returns**

A broker with no registered transports.

```rust
fn new() -> Self
```

