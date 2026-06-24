# pamoja-modbus::function

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

Modbus function codes and the exception codes a device returns when it refuses.

## enum `Function`

A Modbus function code, naming the operation a request asks for.

This enum covers the function codes that read and write the four Modbus data tables
(coils, discrete inputs, holding registers, input registers), which is what the great
majority of field devices use. A function code outside this set still travels fine
through [`Pdu::raw`](crate::Pdu::raw) and [`Adu`](crate::Adu); this enum is the typed
view of the common ones, not a limit on what the framing carries.

**Examples**

```
use pamoja_modbus::Function;

assert_eq!(Function::ReadHoldingRegisters.code(), 0x03);
assert_eq!(Function::from_code(0x10), Some(Function::WriteMultipleRegisters));
assert_eq!(Function::from_code(0x99), None);
```

- `ReadCoils` - Read one or more coils (read/write bits). Function code `0x01`.
- `ReadDiscreteInputs` - Read one or more discrete inputs (read-only bits). Function code `0x02`.
- `ReadHoldingRegisters` - Read one or more holding registers (read/write 16-bit words). Function code `0x03`.
- `ReadInputRegisters` - Read one or more input registers (read-only 16-bit words). Function code `0x04`.
- `WriteSingleCoil` - Write a single coil. Function code `0x05`.
- `WriteSingleRegister` - Write a single holding register. Function code `0x06`.
- `WriteMultipleCoils` - Write a contiguous block of coils. Function code `0x0F`.
- `WriteMultipleRegisters` - Write a contiguous block of holding registers. Function code `0x10`.

### `Function::code`

Returns the wire byte for this function.

**Returns**

The function code as it appears as the first byte of a PDU.

```rust
fn code(self) -> u8
```

### `Function::from_code`

Returns the function a wire byte names, if this crate models it.

**Arguments**

* `code` - the function code byte from the start of a PDU.

**Returns**

The matching [`Function`], or [`None`] for a code this enum does not name
(including the exception responses, whose high bit is set).

```rust
fn from_code(code: u8) -> Option <Function>
```

## enum `Exception`

A Modbus exception code: the reason a device gives for refusing a request.

A device that cannot serve a request replies with the request's function code with
its high bit set, followed by one of these codes. [`Adu::exception`](crate::Adu::exception)
and [`Response::exception`](crate::Response::exception) surface it.

**Examples**

```
use pamoja_modbus::Exception;

assert_eq!(Exception::IllegalDataAddress.code(), 0x02);
assert_eq!(Exception::from_code(0x01), Some(Exception::IllegalFunction));
```

- `IllegalFunction` - The function code is not allowed for this device. Exception code `0x01`.
- `IllegalDataAddress` - The data address is not allowed for this device. Exception code `0x02`.
- `IllegalDataValue` - A value in the request is not allowed for this device. Exception code `0x03`.
- `ServerDeviceFailure` - The device failed while serving the request. Exception code `0x04`.
- `Acknowledge` - The device accepted a long-running request and is still processing it. Exception code `0x05`.
- `ServerDeviceBusy` - The device is busy with a long-running request; retry later. Exception code `0x06`.
- `MemoryParityError` - The device detected a parity error in its memory. Exception code `0x08`.
- `GatewayPathUnavailable` - A gateway could not route the request to the target path. Exception code `0x0A`.
- `GatewayTargetFailedToRespond` - A gateway reached the target device but got no response. Exception code `0x0B`.

### `Exception::code`

Returns the wire byte for this exception.

**Returns**

The exception code as it appears after the function code in an exception response.

```rust
fn code(self) -> u8
```

### `Exception::from_code`

Returns the exception a wire byte names, if it is a defined code.

**Arguments**

* `code` - the exception code byte following the function code.

**Returns**

The matching [`Exception`], or [`None`] for a code this enum does not name.

```rust
fn from_code(code: u8) -> Option <Exception>
```

