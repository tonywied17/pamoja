# pamoja-modbus::response

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

Reading values back out of a Modbus response PDU.

## struct `Response`

A borrowed view over a response PDU, for reading the values a device returned.

A read response is a function code, a byte count, and then the data. [`registers`](Response::registers)
and [`coils`](Response::coils) decode that data into the 16-bit words or the packed
bits it represents; [`exception`](Response::exception) recognises the alternative, a
device that refused the request. The view borrows the PDU and copies nothing.

**Examples**

```
use pamoja_modbus::Response;

// A read-holding-registers reply: function 0x03, byte count 6, three registers.
let pdu = [0x03, 0x06, 0x02, 0x2B, 0x00, 0x00, 0x00, 0x64];
let values: Vec<u16> = Response::new(&pdu).registers().unwrap().collect();
assert_eq!(values, [0x022B, 0x0000, 0x0064]);
```

### `Response <'a>::new`

Wraps a response PDU for reading.

**Arguments**

* `pdu` - the response PDU, a function code followed by its data.

**Returns**

The response view.

```rust
fn new(pdu: &'a [u8]) -> Self
```

### `Response <'a>::function_code`

Returns the function code, the first byte of the PDU.

**Returns**

The function code, or `0` if the PDU is empty.

```rust
fn function_code(&self) -> u8
```

### `Response <'a>::exception`

Returns the exception a device reported, if this is an exception response.

**Returns**

The [`Exception`] if the function code's high bit is set and a defined exception
byte follows it, otherwise [`None`].

```rust
fn exception(&self) -> Option <Exception>
```

### `Response <'a>::registers`

Reads the 16-bit registers from a read-registers response.

**Returns**

An iterator over the registers in order, each decoded from its big-endian pair.

**Errors**

Returns [`ModbusError::MalformedResponse`] if the PDU is truncated, its declared
byte count does not match its data, or that data is not a whole number of registers.

```rust
fn registers(&self) -> Result <Registers <'a>, ModbusError>
```

### `Response <'a>::coils`

Reads the coils or discrete inputs from a read-bits response.

The response packs the bits least-significant first; this unpacks exactly `count`
of them and ignores the padding in the final byte.

**Arguments**

* `count` - how many bits to read, the quantity the request asked for.

**Returns**

An iterator over `count` bits in order.

**Errors**

Returns [`ModbusError::MalformedResponse`] if the PDU is truncated or its declared
byte count does not match the data or the requested `count`.

```rust
fn coils(&self, count: u16) -> Result <Coils <'a>, ModbusError>
```

## struct `Registers`

An iterator over the 16-bit registers of a read-registers response.

## struct `Coils`

An iterator over the bits of a read-coils or read-discrete-inputs response.

