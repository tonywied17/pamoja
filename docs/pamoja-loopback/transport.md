# pamoja-loopback::transport

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

The loopback transport itself.

## struct `Message`

A message delivered over a loopback subscription.

Fields:

- `topic: String` - The topic the message was published to.
- `payload: Vec <u8>` - The raw payload bytes.

## struct `LoopbackTransport`

An in-process transport that routes through a shared [`LoopbackBroker`].

A transport is created disconnected; [`connect`](Transport::connect) registers
it with the broker so it can publish and receive. Inbound messages are read
with [`recv`](LoopbackTransport::recv).

**Examples**

```
use pamoja_core::Transport;
use pamoja_loopback::{LoopbackBroker, LoopbackTransport};

let broker = LoopbackBroker::new();
let mut subscriber = LoopbackTransport::new(broker.clone());
let mut publisher = LoopbackTransport::new(broker);
subscriber.connect().await?;
publisher.connect().await?;

subscriber.subscribe("sensors/+/temperature").await?;
publisher.send("sensors/1/temperature", b"21.5").await?;

let message = subscriber.recv().await?.expect("a message");
assert_eq!(message.topic, "sensors/1/temperature");
assert_eq!(message.payload, b"21.5");
```

### `LoopbackTransport::new`

Creates a disconnected transport bound to `broker`.

**Arguments**

* `broker` - the shared broker this transport publishes to and receives from.

**Returns**

A disconnected transport ready for [`connect`](Transport::connect).

```rust
fn new(broker: LoopbackBroker) -> Self
```

### `LoopbackTransport::is_connected`

Reports whether the transport is connected to its broker.

**Returns**

`true` once [`connect`](Transport::connect) has succeeded and before
[`disconnect`](LoopbackTransport::disconnect) is called.

```rust
fn is_connected(&self) -> bool
```

### `LoopbackTransport::recv`

Awaits the next message from any subscribed topic.

**Returns**

`Some(message)` for the next message, or `None` once the broker and all
other transports have been dropped.

**Errors**

Returns [`Error::Closed`](pamoja_core::Error::Closed) if the transport is
not connected.

```rust
async fn recv(&mut self) -> Result <Option <Message>>
```

### `LoopbackTransport::disconnect`

Disconnects the transport from the broker.

Its registration is pruned from the broker on the next publish.

```rust
fn disconnect(&mut self)
```

