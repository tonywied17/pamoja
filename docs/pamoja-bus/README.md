# pamoja-bus

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

An in-memory typed publish/subscribe event bus.

[`BroadcastBus`] implements the core [`EventBus`](pamoja_core::EventBus) trait
over a bounded broadcast channel: every event published is delivered to every
current subscriber. Producers such as sensors and transports publish events,
and consumers await them, all statically typed to one event type per bus.

The bus is bounded, so a subscriber that falls far enough behind drops the
events it missed and resumes from the most recent ones. This keeps a slow
consumer from holding memory without bound, which matters on constrained
devices.

## struct `BroadcastBus`

A typed publish/subscribe bus that broadcasts each event to all subscribers.

Every handle can both publish and receive. Use [`subscribe`](BroadcastBus::subscribe)
to add an independent consumer; an event published after a handle subscribes
is delivered to it. A subscriber only sees events published after it
subscribed, mirroring a live pub/sub channel.

**Examples**

```
use pamoja_core::EventBus;
use pamoja_bus::BroadcastBus;

let bus = BroadcastBus::new(16);
let mut subscriber = bus.subscribe();
bus.publish("reading").await?;
assert_eq!(subscriber.next_event().await?, Some("reading"));
```

### `BroadcastBus <E>::new`

Creates a bus buffering up to `capacity` unread events per subscriber.

**Arguments**

* `capacity` - the per-subscriber buffer depth; a subscriber further behind
  than this drops the events it missed. Values below one are raised to one.

**Returns**

A bus with one handle that can publish and receive.

```rust
fn new(capacity: usize) -> Self
```

### `BroadcastBus <E>::subscribe`

Creates another handle to the same bus with its own independent subscription.

**Returns**

A handle that receives events published after this call and can also publish.

```rust
fn subscribe(&self) -> Self
```

