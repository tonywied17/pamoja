# pamoja-zenoh::transport

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

The live Zenoh transport: a Zenoh session behind the core [`Transport`] trait.

[`ZenohTransport`] opens a Zenoh session and exposes it through the protocol-agnostic
[`Transport`](pamoja_core::Transport) surface, so Zenoh serves as the efficient edge-to-edge and
fleet transport alongside MQTT, CoAP, and the radios. Like the other live transports it owns a
background task per subscription that forwards samples into a queue [`recv`](ZenohTransport::recv)
drains.

## struct `Message`

A sample received from a subscribed key expression.

Fields:

- `key: String` - The key expression the sample was published to.
- `payload: Vec <u8>` - The raw payload bytes.

## struct `ZenohConfig`

Connection settings for a [`ZenohTransport`].

The default is a Zenoh peer that discovers others by multicast scouting. The chained setters
pin explicit endpoints for a deterministic link, which is what fleets on routed networks use.

### `ZenohConfig::new`

Creates a default configuration: a peer using multicast scouting.

**Returns**

The configuration.

```rust
fn new() -> Self
```

### `ZenohConfig::listen_on`

Adds a Zenoh endpoint to listen on, for example `tcp/0.0.0.0:7447`.

**Arguments**

* `endpoint` - the Zenoh locator to accept connections on.

**Returns**

The updated configuration, for chaining.

```rust
fn listen_on(mut self, endpoint: &str) -> Self
```

### `ZenohConfig::connect_to`

Adds a Zenoh endpoint to connect to, for example `tcp/192.168.1.10:7447`.

**Arguments**

* `endpoint` - the Zenoh locator of a peer or router to dial.

**Returns**

The updated configuration, for chaining.

```rust
fn connect_to(mut self, endpoint: &str) -> Self
```

### `ZenohConfig::multicast_scouting`

Enables or disables multicast scouting (peer auto-discovery).

**Arguments**

* `enabled` - whether to discover peers by multicast; turn off on networks that block it or
  when using explicit endpoints.

**Returns**

The updated configuration, for chaining.

```rust
fn multicast_scouting(mut self, enabled: bool) -> Self
```

### `ZenohConfig::into_zenoh`

Consumes the wrapper and returns the underlying Zenoh configuration.

**Returns**

The Zenoh [`Config`].

```rust
fn into_zenoh(self) -> Config
```

## struct `ZenohTransport`

A Zenoh session that implements the core [`Transport`] trait.

Created disconnected; [`connect`](Transport::connect) opens the session. Each
[`subscribe`](Transport::subscribe) declares a Zenoh subscriber and spawns a task forwarding its
samples to an internal queue, and [`recv`](ZenohTransport::recv) awaits the next one.

### `ZenohTransport::new`

Creates a transport from the given configuration without connecting.

**Arguments**

* `config` - the session settings.

**Returns**

A disconnected transport ready for [`connect`](Transport::connect).

```rust
fn new(config: ZenohConfig) -> Self
```

### `ZenohTransport::is_connected`

Reports whether the session is currently open.

**Returns**

`true` once [`connect`](Transport::connect) has succeeded.

```rust
fn is_connected(&self) -> bool
```

### `ZenohTransport::recv`

Awaits the next sample from any subscribed key expression.

**Returns**

`Some(message)` for the next queued sample, or `None` once every subscription has ended.

**Errors**

Returns [`Error::Closed`](pamoja_core::Error::Closed) if the transport is not connected.

```rust
async fn recv(&mut self) -> Result <Option <Message>>
```

### `ZenohTransport::disconnect`

Closes the session and stops the subscription tasks.

**Returns**

`Ok(())` once the session has been closed.

**Errors**

Best-effort teardown that does not surface session-close errors, so it returns `Ok(())`.

```rust
async fn disconnect(&mut self) -> Result <()>
```

