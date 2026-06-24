# pamoja-lorawan::session

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

The activated session and the data frames it secures.

## struct `Session`

An activated LoRaWAN session: a device address and the two session keys.

This is the state a device holds once it is activated, whether by personalization
(the address and keys provisioned directly) or by a join exchange. It secures every
data frame: the network session key authenticates the whole frame through its MIC, and
the application session key encrypts the payload, with the device address and frame
counter folded into both so a frame is bound to its place in the stream.

**Examples**

```
use pamoja_lorawan::{Session, Uplink};

let session = Session::new(0x2601_1BDA, [0x11; 16], [0x22; 16]);
let frame = session.encode_uplink(&Uplink::new(1, 1, b"hello")).unwrap();

// The receiver, holding the same session, recovers the payload.
let rx = session.decode(frame.as_bytes(), 1).unwrap();
assert_eq!(rx.payload(), b"hello");
```

### `Session::new`

Creates a session from a device address and its two session keys.

**Arguments**

* `dev_addr` - the device address the network assigned.
* `nwk_skey` - the network session key, which authenticates frames.
* `app_skey` - the application session key, which encrypts payloads.

**Returns**

The session.

```rust
fn new(dev_addr: u32, nwk_skey: [u8 ; 16], app_skey: [u8 ; 16]) -> Self
```

### `Session::dev_addr`

Returns the device address this session is bound to.

**Returns**

The device address.

```rust
fn dev_addr(&self) -> u32
```

### `Session::encode_uplink`

Encodes an uplink data frame, encrypting the payload and appending the MIC.

**Arguments**

* `uplink` - the uplink to send.

**Returns**

The frame ready for the radio.

**Errors**

Returns [`LorawanError::PayloadTooLong`] if the payload and options do not fit a
single frame.

```rust
fn encode_uplink(&self, uplink: &Uplink) -> Result <PhyPayload, LorawanError>
```

### `Session::encode_downlink`

Encodes a downlink data frame, encrypting the payload and appending the MIC.

**Arguments**

* `downlink` - the downlink to send.

**Returns**

The frame ready for the radio.

**Errors**

Returns [`LorawanError::PayloadTooLong`] if the payload and options do not fit a
single frame.

```rust
fn encode_downlink(&self, downlink: &Downlink) -> Result <PhyPayload, LorawanError>
```

### `Session::decode`

Decodes a received data frame: verifies the MIC, then decrypts the payload.

**Arguments**

* `bytes` - the raw frame as it came off the radio.
* `fcnt` - the full 32-bit frame counter expected for this frame; its low 16 bits
  must match the counter the frame carries.

**Returns**

The decoded frame, with its payload decrypted.

**Errors**

Returns [`LorawanError::FrameTooShort`] if the frame is too small,
[`LorawanError::UnsupportedMType`] if it is not a data frame,
[`LorawanError::FcntMismatch`] if the counter does not match, or
[`LorawanError::MicMismatch`] if the MIC does not verify.

```rust
fn decode(&self, bytes: &[u8], fcnt: u32) -> Result <RxData, LorawanError>
```

## struct `Uplink`

An uplink data frame to encode, built up from the fields a sender sets.

Construct one with [`new`](Uplink::new) and turn on whatever applies; the rest default
off. A higher port carries application data; port `0` carries MAC commands.

**Examples**

```
use pamoja_lorawan::Uplink;

let uplink = Uplink::new(7, 2, b"reading").confirmed().with_adr();
```

### `Uplink <'a>::new`

Creates an unconfirmed uplink with no options set.

**Arguments**

* `fcnt` - the frame counter for this uplink.
* `fport` - the port; `0` for MAC commands, otherwise an application port.
* `payload` - the application payload to carry.

**Returns**

The uplink.

```rust
fn new(fcnt: u32, fport: u8, payload: &'a [u8]) -> Self
```

### `Uplink <'a>::confirmed`

Marks the uplink as confirmed, asking the network to acknowledge it.

**Returns**

The uplink, for chaining.

```rust
fn confirmed(mut self) -> Self
```

### `Uplink <'a>::with_adr`

Sets the adaptive-data-rate bit, letting the network manage the data rate.

**Returns**

The uplink, for chaining.

```rust
fn with_adr(mut self) -> Self
```

### `Uplink <'a>::with_ack`

Sets the acknowledgement bit, confirming a previously received downlink.

**Returns**

The uplink, for chaining.

```rust
fn with_ack(mut self) -> Self
```

### `Uplink <'a>::with_fopts`

Carries MAC command options in the frame header.

**Arguments**

* `fopts` - the frame options, up to 15 bytes.

**Returns**

The uplink, for chaining.

```rust
fn with_fopts(mut self, fopts: &'a [u8]) -> Self
```

## struct `Downlink`

A downlink data frame to encode, built up from the fields a sender sets.

Construct one with [`new`](Downlink::new) and turn on whatever applies; the rest
default off.

### `Downlink <'a>::new`

Creates an unconfirmed downlink with no options set.

**Arguments**

* `fcnt` - the frame counter for this downlink.
* `fport` - the port; `0` for MAC commands, otherwise an application port.
* `payload` - the application payload to carry.

**Returns**

The downlink.

```rust
fn new(fcnt: u32, fport: u8, payload: &'a [u8]) -> Self
```

### `Downlink <'a>::confirmed`

Marks the downlink as confirmed, asking the device to acknowledge it.

**Returns**

The downlink, for chaining.

```rust
fn confirmed(mut self) -> Self
```

### `Downlink <'a>::with_adr`

Sets the adaptive-data-rate bit.

**Returns**

The downlink, for chaining.

```rust
fn with_adr(mut self) -> Self
```

### `Downlink <'a>::with_ack`

Sets the acknowledgement bit, confirming a previously received uplink.

**Returns**

The downlink, for chaining.

```rust
fn with_ack(mut self) -> Self
```

### `Downlink <'a>::with_fpending`

Sets the frame-pending bit, signalling more downlinks are waiting.

**Returns**

The downlink, for chaining.

```rust
fn with_fpending(mut self) -> Self
```

### `Downlink <'a>::with_fopts`

Carries MAC command options in the frame header.

**Arguments**

* `fopts` - the frame options, up to 15 bytes.

**Returns**

The downlink, for chaining.

```rust
fn with_fopts(mut self, fopts: &'a [u8]) -> Self
```

## struct `RxData`

A decoded data frame, with its payload decrypted.

What [`Session::decode`] returns once a frame's MIC has verified: the header fields and
the recovered payload, held in fixed buffers.

### `RxData::direction`

Returns the direction the frame travelled.

**Returns**

[`Direction::Uplink`] or [`Direction::Downlink`].

```rust
fn direction(&self) -> Direction
```

### `RxData::dev_addr`

Returns the device address the frame carried.

**Returns**

The device address.

```rust
fn dev_addr(&self) -> u32
```

### `RxData::fcnt`

Returns the low 16 bits of the frame counter the frame carried.

**Returns**

The frame counter's low half.

```rust
fn fcnt(&self) -> u16
```

### `RxData::confirmed`

Reports whether the frame is a confirmed frame that expects an acknowledgement.

**Returns**

`true` for a confirmed frame.

```rust
fn confirmed(&self) -> bool
```

### `RxData::adr`

Reports whether the adaptive-data-rate bit is set.

**Returns**

`true` if the bit is set.

```rust
fn adr(&self) -> bool
```

### `RxData::ack`

Reports whether the acknowledgement bit is set.

**Returns**

`true` if the bit is set.

```rust
fn ack(&self) -> bool
```

### `RxData::fpending`

Reports whether the frame-pending bit is set (downlink only).

**Returns**

`true` if the bit is set.

```rust
fn fpending(&self) -> bool
```

### `RxData::fport`

Returns the port the frame was sent on, if it carried a port and payload.

**Returns**

The port, or [`None`] for a frame with no port or payload.

```rust
fn fport(&self) -> Option <u8>
```

### `RxData::fopts`

Returns the frame options carried in the header.

**Returns**

The frame option bytes, which may be empty.

```rust
fn fopts(&self) -> &[u8]
```

### `RxData::payload`

Returns the decrypted payload.

**Returns**

The application payload, which may be empty.

```rust
fn payload(&self) -> &[u8]
```

