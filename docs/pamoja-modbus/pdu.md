# pamoja-modbus::pdu

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

The Modbus protocol data unit and the standard requests that build one.

## struct `Pdu`

A Modbus protocol data unit: a function code followed by its data.

The PDU is the part of a frame that is the same on every transport. On RTU it sits
between the unit address and the CRC; wrap one with [`to_adu`](Pdu::to_adu) to get a
frame ready for the wire.

The constructors build the standard requests so callers state intent ("read three
holding registers") rather than packing bytes, encoding addresses and counts in the
big-endian order Modbus uses. For a function code this crate does not name, [`raw`](Pdu::raw)
carries arbitrary bytes through unchanged. The data is held in a fixed buffer, so a
PDU needs no allocation.

**Examples**

```
use pamoja_modbus::Pdu;

let pdu = Pdu::write_single_register(0x0001, 0x0003);
assert_eq!(pdu.as_bytes(), &[0x06, 0x00, 0x01, 0x00, 0x03]);
```

### `Pdu::read_coils`

Builds a read-coils request (function `0x01`).

**Arguments**

* `start` - the address of the first coil to read.
* `count` - how many coils to read.

**Returns**

The request PDU.

```rust
fn read_coils(start: u16, count: u16) -> Pdu
```

### `Pdu::read_discrete_inputs`

Builds a read-discrete-inputs request (function `0x02`).

**Arguments**

* `start` - the address of the first discrete input to read.
* `count` - how many inputs to read.

**Returns**

The request PDU.

```rust
fn read_discrete_inputs(start: u16, count: u16) -> Pdu
```

### `Pdu::read_holding_registers`

Builds a read-holding-registers request (function `0x03`).

**Arguments**

* `start` - the address of the first holding register to read.
* `count` - how many registers to read.

**Returns**

The request PDU.

```rust
fn read_holding_registers(start: u16, count: u16) -> Pdu
```

### `Pdu::read_input_registers`

Builds a read-input-registers request (function `0x04`).

**Arguments**

* `start` - the address of the first input register to read.
* `count` - how many registers to read.

**Returns**

The request PDU.

```rust
fn read_input_registers(start: u16, count: u16) -> Pdu
```

### `Pdu::write_single_coil`

Builds a write-single-coil request (function `0x05`).

**Arguments**

* `address` - the address of the coil to write.
* `on` - the value to write: `true` drives the coil on, `false` off.

**Returns**

The request PDU.

```rust
fn write_single_coil(address: u16, on: bool) -> Pdu
```

### `Pdu::write_single_register`

Builds a write-single-register request (function `0x06`).

**Arguments**

* `address` - the address of the holding register to write.
* `value` - the 16-bit value to write.

**Returns**

The request PDU.

```rust
fn write_single_register(address: u16, value: u16) -> Pdu
```

### `Pdu::write_multiple_registers`

Builds a write-multiple-registers request (function `0x10`).

**Arguments**

* `start` - the address of the first holding register to write.
* `values` - the 16-bit values to write to consecutive registers.

**Returns**

The request PDU.

**Errors**

Returns [`ModbusError::InvalidValueCount`] if `values` is empty or holds more than
[`MAX_WRITE_REGISTERS`](Pdu::MAX_WRITE_REGISTERS) values.

```rust
fn write_multiple_registers(start: u16, values: &[u16]) -> Result <Pdu, ModbusError>
```

### `Pdu::write_multiple_coils`

Builds a write-multiple-coils request (function `0x0F`).

The coils are packed into bytes least-significant bit first, the order Modbus
uses; any unused bits in the final byte are left zero.

**Arguments**

* `start` - the address of the first coil to write.
* `values` - the coil states to write, one `bool` per coil.

**Returns**

The request PDU.

**Errors**

Returns [`ModbusError::InvalidValueCount`] if `values` is empty or holds more than
[`MAX_WRITE_COILS`](Pdu::MAX_WRITE_COILS) values.

```rust
fn write_multiple_coils(start: u16, values: &[bool]) -> Result <Pdu, ModbusError>
```

### `Pdu::raw`

Builds a PDU from a raw function code and data, the escape hatch for function
codes this crate does not name.

**Arguments**

* `function` - the function code byte.
* `data` - the bytes that follow it, used verbatim.

**Returns**

The PDU.

**Errors**

Returns [`ModbusError::FrameTooLong`] if the function code plus `data` would not
fit a PDU (more than [`MAX_LEN`](Pdu::MAX_LEN) bytes).

```rust
fn raw(function: u8, data: &[u8]) -> Result <Pdu, ModbusError>
```

### `Pdu::function_code`

Returns the function code, the first byte of the PDU.

**Returns**

The function code.

```rust
fn function_code(&self) -> u8
```

### `Pdu::as_bytes`

Returns the PDU bytes: the function code followed by its data.

**Returns**

The PDU as a byte slice.

```rust
fn as_bytes(&self) -> &[u8]
```

### `Pdu::to_adu`

Wraps this PDU into an RTU frame addressed to a unit, appending the CRC.

**Arguments**

* `address` - the unit (slave) address the frame is for.

**Returns**

The [`Adu`] ready to send.

```rust
fn to_adu(&self, address: u8) -> Adu
```

