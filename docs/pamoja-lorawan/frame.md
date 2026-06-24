# pamoja-lorawan::frame

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

The LoRaWAN PHYPayload and the small shared pieces of its header.

## const `MAX_FRAME`

The largest PHYPayload, in bytes, this crate builds or accepts.

Comfortably above the largest regional LoRaWAN maximum, so a frame always fits.

```rust
const MAX_FRAME: usize
```

## const `MAX_PAYLOAD`

The largest application payload, in bytes, a single frame can carry (with no frame
options present).

```rust
const MAX_PAYLOAD: usize
```

## enum `Direction`

The direction a frame travels, which the MIC and the payload encryption both fold in.

- `Uplink` - From an end device up to the network.
- `Downlink` - From the network down to an end device.

## struct `PhyPayload`

An encoded LoRaWAN frame, the bytes that go on the air.

Built by a [`Session`](crate::Session) or a join exchange, and held in a fixed buffer
so encoding never allocates. [`as_bytes`](PhyPayload::as_bytes) hands the radio exactly
what to transmit.

### `PhyPayload::as_bytes`

Returns the frame as bytes, ready to transmit.

**Returns**

The whole PHYPayload, MIC included.

```rust
fn as_bytes(&self) -> &[u8]
```

