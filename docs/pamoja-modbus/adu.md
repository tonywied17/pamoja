# pamoja-modbus::adu

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

The Modbus RTU application data unit: the frame that goes on the wire.

## struct `Adu`

A Modbus RTU frame: a unit address, a PDU, and a trailing CRC.

This is the complete unit of bytes an RTU transmitter puts on the bus and a receiver
pulls off it. [`from_pdu`](Adu::from_pdu) builds one to send by appending the CRC;
[`parse`](Adu::parse) reads one received, verifying the CRC so a frame corrupted in
transit never reaches the application. The frame lives in a fixed buffer, so neither
path allocates.

**Examples**

```
use pamoja_modbus::Adu;

let frame = Adu::from_pdu(0x11, &[0x03, 0x00, 0x6B, 0x00, 0x03]).unwrap();
assert_eq!(frame.address(), 0x11);
assert_eq!(frame.function_code(), 0x03);

// A receiver validates the same bytes against the CRC they carry.
let received = Adu::parse(frame.as_bytes()).unwrap();
assert_eq!(received.pdu(), &[0x03, 0x00, 0x6B, 0x00, 0x03]);
```

### `Adu::from_pdu`

Builds a frame for a unit address and PDU, appending the CRC.

**Arguments**

* `address` - the unit (slave) address the frame is for.
* `pdu` - the protocol data unit: a function code followed by its data.

**Returns**

The frame ready to send.

**Errors**

Returns [`ModbusError::FrameTooLong`] if `pdu` is longer than a PDU may be, so the
frame would exceed [`MAX_LEN`](Adu::MAX_LEN) bytes.

```rust
fn from_pdu(address: u8, pdu: &[u8]) -> Result <Adu, ModbusError>
```

### `Adu::parse`

Parses a received frame, verifying its CRC.

**Arguments**

* `bytes` - the raw frame as it came off the wire, CRC included.

**Returns**

The validated frame.

**Errors**

Returns [`ModbusError::FrameTooShort`] if `bytes` is shorter than a valid frame,
[`ModbusError::FrameTooLong`] if it is longer than [`MAX_LEN`](Adu::MAX_LEN), or
[`ModbusError::CrcMismatch`] if the trailing CRC does not match the contents.

```rust
fn parse(bytes: &[u8]) -> Result <Adu, ModbusError>
```

### `Adu::address`

Returns the unit address, the first byte of the frame.

**Returns**

The unit (slave) address.

```rust
fn address(&self) -> u8
```

### `Adu::function_code`

Returns the function code, the first byte of the PDU.

**Returns**

The function code. An exception response has its high bit set.

```rust
fn function_code(&self) -> u8
```

### `Adu::pdu`

Returns the PDU: the frame without its address and CRC.

**Returns**

The protocol data unit as a byte slice.

```rust
fn pdu(&self) -> &[u8]
```

### `Adu::as_bytes`

Returns the whole frame, CRC included, ready for the wire.

**Returns**

The frame as a byte slice.

```rust
fn as_bytes(&self) -> &[u8]
```

### `Adu::exception`

Returns the exception a device reported, if this frame is an exception response.

**Returns**

The [`Exception`] if the function code's high bit is set and an exception byte
follows it, otherwise [`None`] (including for a defined-but-unknown exception code).

```rust
fn exception(&self) -> Option <Exception>
```

### `Adu::response`

Returns a reader over this frame's PDU for decoding a response.

**Returns**

A [`Response`] borrowing the PDU.

```rust
fn response(&self) -> Response <'_>
```

