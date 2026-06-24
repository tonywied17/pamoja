# pamoja-mesh::frame

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

The mesh frame: the addressed, hop-limited, checksummed packet on the wire.

## const `BROADCAST`

The destination that addresses a frame to every node, for flooding the whole mesh.

```rust
const BROADCAST: u32
```

## struct `Frame`

An addressed mesh packet.

A frame names where it came from and where it is going, carries a sequence number its
origin assigns, counts down a hop limit as it is relayed, and ends with a checksum.
The byte layout is fixed and big-endian:

```text
0       version
1..=4   source node       (u32)
5..=8   destination node  (u32, BROADCAST for every node)
9..=10  sequence id       (u16)
11      hop limit
12..    payload
last 2  checksum          (u16)
```

The checksum covers every byte except the hop limit, which changes at each relay. So
the check is end to end: a node can confirm a flooded packet's payload is intact no
matter how many relays forwarded it, and a relay spends a hop without recomputing it.
The whole frame lives in a fixed buffer, so neither building nor parsing allocates.

**Examples**

```
use pamoja_mesh::Frame;

let frame = Frame::new(0x0A, 0x0B, 7, b"hello").unwrap();
assert_eq!(frame.src(), 0x0A);
assert_eq!(frame.dst(), 0x0B);
assert_eq!(frame.id(), 7);
assert_eq!(frame.payload(), b"hello");

let received = Frame::parse(frame.as_bytes()).unwrap();
assert_eq!(received.payload(), b"hello");
```

### `Frame::new`

Builds a frame from a source, a destination, a sequence id, and a payload, starting
at [`DEFAULT_HOP_LIMIT`](Frame::DEFAULT_HOP_LIMIT).

**Arguments**

* `src` - the origin node's address.
* `dst` - the destination node's address, or [`BROADCAST`] for every node.
* `id` - the sequence number the origin assigns, increasing per message; with the
  source it identifies a packet as it floods, for [`dedup_key`](Frame::dedup_key).
* `payload` - the bytes to carry.

**Returns**

The frame, ready to send.

**Errors**

Returns [`MeshError::PayloadTooLong`] if `payload` is longer than
[`MAX_PAYLOAD`](Frame::MAX_PAYLOAD).

```rust
fn new(src: u32, dst: u32, id: u16, payload: &[u8]) -> Result <Frame, MeshError>
```

### `Frame::broadcast`

Builds a frame addressed to every node, for flooding the whole mesh.

**Arguments**

* `src` - the origin node's address.
* `id` - the sequence number the origin assigns.
* `payload` - the bytes to carry.

**Returns**

The broadcast frame, ready to send.

**Errors**

Returns [`MeshError::PayloadTooLong`] if `payload` is longer than
[`MAX_PAYLOAD`](Frame::MAX_PAYLOAD).

```rust
fn broadcast(src: u32, id: u16, payload: &[u8]) -> Result <Frame, MeshError>
```

### `Frame::with_hop_limit`

Sets the hop limit, the number of further relays the frame is allowed.

The checksum does not cover the hop limit, so this needs no recomputation and
leaves a parsed frame still valid.

**Arguments**

* `hop_limit` - the new hop limit. `0` means no node should relay the frame further.

**Returns**

The frame with the hop limit set, for chaining.

```rust
fn with_hop_limit(mut self, hop_limit: u8) -> Frame
```

### `Frame::parse`

Parses a received frame, verifying its version and checksum.

**Arguments**

* `bytes` - the raw frame as it came off the radio.

**Returns**

The validated frame.

**Errors**

Returns [`MeshError::FrameTooShort`] or [`MeshError::FrameTooLong`] if the length is
outside a frame's bounds, [`MeshError::UnsupportedVersion`] if the version byte is
not [`VERSION`](Frame::VERSION), or [`MeshError::CrcMismatch`] if the checksum does
not match the contents.

```rust
fn parse(bytes: &[u8]) -> Result <Frame, MeshError>
```

### `Frame::version`

Returns the protocol version.

**Returns**

The version byte.

```rust
fn version(&self) -> u8
```

### `Frame::src`

Returns the source node's address.

**Returns**

The address of the node that originated the frame.

```rust
fn src(&self) -> u32
```

### `Frame::dst`

Returns the destination node's address.

**Returns**

The address of the destination node, or [`BROADCAST`] for every node.

```rust
fn dst(&self) -> u32
```

### `Frame::id`

Returns the sequence id the origin assigned.

**Returns**

The sequence number.

```rust
fn id(&self) -> u16
```

### `Frame::hop_limit`

Returns the remaining hop limit.

**Returns**

The number of further relays the frame is allowed.

```rust
fn hop_limit(&self) -> u8
```

### `Frame::payload`

Returns the payload.

**Returns**

The carried bytes, without the header or checksum.

```rust
fn payload(&self) -> &[u8]
```

### `Frame::as_bytes`

Returns the whole frame, checksum included, ready for the radio.

**Returns**

The frame as a byte slice.

```rust
fn as_bytes(&self) -> &[u8]
```

### `Frame::is_broadcast`

Reports whether the frame is addressed to every node.

**Returns**

`true` if the destination is [`BROADCAST`].

```rust
fn is_broadcast(&self) -> bool
```

### `Frame::dedup_key`

Returns the key that identifies this packet as it floods: its source and sequence
id.

**Returns**

The `(source, id)` pair, for a [`SeenCache`](crate::SeenCache).

```rust
fn dedup_key(&self) ->(u32, u16)
```

### `Frame::relayed`

Returns the frame to forward one hop further, with a hop spent.

**Returns**

The same frame with its hop limit reduced by one, or [`None`] if the hop limit is
already `0` and the frame must not be relayed further.

```rust
fn relayed(&self) -> Option <Frame>
```

