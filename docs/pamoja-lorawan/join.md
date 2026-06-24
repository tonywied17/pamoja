# pamoja-lorawan::join

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

Over-the-air activation: the join exchange that turns root keys into a session.

## struct `Device`

An end device's root credentials for over-the-air activation.

Where a [`Session`] is the state of an already-activated device, a `Device` holds what
it takes to activate: the device and application identifiers and the application root
key. It builds the [`join_request`](Device::join_request) a device broadcasts and turns
the network's reply into a ready [`Session`] with [`accept_join`](Device::accept_join),
deriving the session keys the spec prescribes.

The 8-byte identifiers are given most-significant byte first, as they are written; the
join-request transmits them little-endian, as the spec requires.

### `Device::new`

Creates a device from its identifiers and application root key.

**Arguments**

* `dev_eui` - the device identifier, most-significant byte first.
* `app_eui` - the application (join) identifier, most-significant byte first.
* `app_key` - the application root key.

**Returns**

The device.

```rust
fn new(dev_eui: [u8 ; 8], app_eui: [u8 ; 8], app_key: [u8 ; 16]) -> Self
```

### `Device::join_request`

Builds a join-request to broadcast.

**Arguments**

* `dev_nonce` - a nonce the device must not reuse; keep it for the matching
  [`accept_join`](Device::accept_join), which needs it to derive the keys.

**Returns**

The join-request frame.

```rust
fn join_request(&self, dev_nonce: u16) -> PhyPayload
```

### `Device::accept_join`

Accepts a join-accept, deriving the activated session.

Decrypts the reply, verifies its MIC against the application root key, and derives
the network and application session keys from the nonces it carries.

**Arguments**

* `bytes` - the raw join-accept as it came off the radio.
* `dev_nonce` - the same nonce passed to the [`join_request`](Device::join_request)
  this reply answers.

**Returns**

The activation, including the ready-to-use [`Session`].

**Errors**

Returns [`LorawanError::FrameTooShort`] or [`LorawanError::MalformedFrame`] if the
reply is not a valid join-accept shape, [`LorawanError::UnsupportedMType`] if it is
not a join-accept, or [`LorawanError::MicMismatch`] if its MIC does not verify.

```rust
fn accept_join(&self, bytes: &[u8], dev_nonce: u16) -> Result <JoinAccept, LorawanError>
```

## struct `JoinAccept`

A successful activation: the session to use, plus the network parameters the accept
carried.

### `JoinAccept::session`

Returns the activated session, ready to secure data frames.

**Returns**

The [`Session`].

```rust
fn session(&self) -> Session
```

### `JoinAccept::dev_addr`

Returns the device address the network assigned.

**Returns**

The device address.

```rust
fn dev_addr(&self) -> u32
```

### `JoinAccept::net_id`

Returns the network identifier, a 24-bit value.

**Returns**

The NetID.

```rust
fn net_id(&self) -> u32
```

### `JoinAccept::dl_settings`

Returns the downlink settings byte, which selects the downlink data rates.

**Returns**

The DLSettings byte.

```rust
fn dl_settings(&self) -> u8
```

### `JoinAccept::rx_delay`

Returns the delay, in seconds, before the first receive window.

**Returns**

The RxDelay value.

```rust
fn rx_delay(&self) -> u8
```

