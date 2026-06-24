# pamoja-ffi::mqtt

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

The C ABI for the MQTT transport.

These functions wrap [`pamoja_mqtt`] for callers that reach the SDK through
the flat C boundary. Because that boundary has no async support, the crate
owns a single multi-threaded Tokio runtime and each call blocks on it until
the underlying async operation completes; a host that wants concurrency runs
these calls on its own threads. The shared transport sits behind an async
mutex, mirroring the Node and Python bindings so behavior matches across
languages.

## enum `PamojaQos`

MQTT delivery guarantee, mirroring the protocol's quality-of-service levels.

- `AtMostOnce` - Fire and forget; the broker does not acknowledge delivery.
- `AtLeastOnce` - Delivered at least once and acknowledged.
- `ExactlyOnce` - Delivered exactly once via a four-step handshake.

## struct `PamojaMqttConfig`

Connection settings for an MQTT client.

`client_id` and `host` are borrowed null-terminated UTF-8 strings. A
`keep_alive_secs` or `capacity` of `0` selects the core default.

Fields:

- `client_id: * const c_char` - The MQTT client identifier presented to the broker.
- `host: * const c_char` - The broker hostname or IP address.
- `port: u16` - The broker TCP port, conventionally 1883 for plaintext MQTT.
- `keep_alive_secs: u32` - Keep-alive interval in seconds, or 0 for the default of 30.
- `capacity: u32` - Bound on outstanding client requests, or 0 for the default of 64.
- `qos: PamojaQos` - Default quality of service for publishes and subscriptions.

## struct `PamojaMqttClient`

An opaque handle to an MQTT client transport.

## struct `PamojaMqttMessage`

An opaque handle to a message received from a subscribed topic.

## fn `pamoja_mqtt_client_new`

Creates a disconnected MQTT client from the given settings.

**Returns**

A heap-allocated client handle the caller owns and must release with
[`pamoja_mqtt_client_free`], or null on failure with the reason available from
[`pamoja_last_error_message`](crate::pamoja_last_error_message).

**Safety**

`config` must point to a valid [`PamojaMqttConfig`] whose `client_id` and
`host` are valid null-terminated UTF-8 strings for the duration of the call.

```rust
unsafe extern "C" fn pamoja_mqtt_client_new(config: * const PamojaMqttConfig,) -> * mut PamojaMqttClient
```

## fn `pamoja_mqtt_client_connect`

Connects to the broker and starts the background event loop.

**Returns**

[`PamojaStatus::Ok`] once connected, or an error status whose message is
available from [`pamoja_last_error_message`](crate::pamoja_last_error_message).

**Safety**

`client` must be a non-null handle returned by [`pamoja_mqtt_client_new`] and
not yet freed.

```rust
unsafe extern "C" fn pamoja_mqtt_client_connect(client: * mut PamojaMqttClient) -> PamojaStatus
```

## fn `pamoja_mqtt_client_publish`

Publishes a payload to a topic.

**Returns**

[`PamojaStatus::Ok`] once the payload is handed to the transport, or an error
status.

**Safety**

`client` must be a live handle from [`pamoja_mqtt_client_new`]; `topic` must be
a valid null-terminated UTF-8 string; and `payload` must point to at least
`payload_len` bytes, or be null when `payload_len` is 0.

```rust
unsafe extern "C" fn pamoja_mqtt_client_publish(client: * mut PamojaMqttClient, topic: * const c_char, payload: * const u8, payload_len: usize,) -> PamojaStatus
```

## fn `pamoja_mqtt_client_subscribe`

Subscribes to a topic filter.

**Returns**

[`PamojaStatus::Ok`] once the subscription is registered, or an error status.

**Safety**

`client` must be a live handle from [`pamoja_mqtt_client_new`] and `topic` a
valid null-terminated UTF-8 string.

```rust
unsafe extern "C" fn pamoja_mqtt_client_subscribe(client: * mut PamojaMqttClient, topic: * const c_char,) -> PamojaStatus
```

## fn `pamoja_mqtt_client_recv`

Awaits the next message from any subscribed topic.

On success `*out_message` is set to a new message handle the caller owns and
must release with [`pamoja_mqtt_message_free`], or to null once the connection
has ended and no further messages will arrive.

**Returns**

[`PamojaStatus::Ok`] on success (including end of stream), or an error status.

**Safety**

`client` must be a live handle from [`pamoja_mqtt_client_new`] and
`out_message` must point to a writable `*mut PamojaMqttMessage`.

```rust
unsafe extern "C" fn pamoja_mqtt_client_recv(client: * mut PamojaMqttClient, out_message: * mut * mut PamojaMqttMessage,) -> PamojaStatus
```

## fn `pamoja_mqtt_client_is_connected`

Reports whether the client currently holds an active connection.

**Returns**

`true` while connected. Returns `false` for a null handle or if the check
panics.

**Safety**

`client` must be a live handle from [`pamoja_mqtt_client_new`], or null.

```rust
unsafe extern "C" fn pamoja_mqtt_client_is_connected(client: * mut PamojaMqttClient) -> bool
```

## fn `pamoja_mqtt_client_disconnect`

Closes the connection and stops the background event loop.

**Returns**

[`PamojaStatus::Ok`] once the client has disconnected.

**Safety**

`client` must be a live handle from [`pamoja_mqtt_client_new`].

```rust
unsafe extern "C" fn pamoja_mqtt_client_disconnect(client: * mut PamojaMqttClient,) -> PamojaStatus
```

## fn `pamoja_mqtt_client_free`

Releases an MQTT client handle.

Passing null is a no-op.

**Safety**

`client` must be a handle from [`pamoja_mqtt_client_new`] that has not already
been freed, or null. After this call the handle must not be used again.

```rust
unsafe extern "C" fn pamoja_mqtt_client_free(client: * mut PamojaMqttClient)
```

## fn `pamoja_mqtt_message_topic`

Returns the topic a message was published to.

**Returns**

A pointer to a null-terminated UTF-8 string valid until the message is freed,
or null if `message` is null.

**Safety**

`message` must be a live handle from [`pamoja_mqtt_client_recv`], or null.

```rust
unsafe extern "C" fn pamoja_mqtt_message_topic(message: * const PamojaMqttMessage,) -> * const c_char
```

## fn `pamoja_mqtt_message_payload`

Returns a pointer to a message's payload bytes.

Use [`pamoja_mqtt_message_payload_len`] for the length. The pointer is valid
until the message is freed.

**Returns**

A pointer to the payload bytes, or null if `message` is null.

**Safety**

`message` must be a live handle from [`pamoja_mqtt_client_recv`], or null.

```rust
unsafe extern "C" fn pamoja_mqtt_message_payload(message: * const PamojaMqttMessage,) -> * const u8
```

## fn `pamoja_mqtt_message_payload_len`

Returns the length in bytes of a message's payload.

**Returns**

The payload length, or 0 if `message` is null.

**Safety**

`message` must be a live handle from [`pamoja_mqtt_client_recv`], or null.

```rust
unsafe extern "C" fn pamoja_mqtt_message_payload_len(message: * const PamojaMqttMessage,) -> usize
```

## fn `pamoja_mqtt_message_free`

Releases a message handle.

Passing null is a no-op.

**Safety**

`message` must be a handle from [`pamoja_mqtt_client_recv`] that has not
already been freed, or null. After this call the handle must not be used again.

```rust
unsafe extern "C" fn pamoja_mqtt_message_free(message: * mut PamojaMqttMessage)
```

