# pamoja-gpio::i2c

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

I2C device addressing per the NXP I2C-bus specification (UM10204).

An I2C transfer begins with the controller sending the device's address. The exact
bytes are pinned down by the specification, and they are easy to get subtly wrong: the
7-bit address shares its byte with the read/write bit, so the value a datasheet prints
is not the byte that goes on the wire, and the 10-bit extension spends a reserved
prefix and spreads its bits across two bytes. This module builds those bytes exactly
and rejects an address that is out of range or, on request, one the specification
reserves.

## enum `Direction`

Whether an I2C transfer reads from or writes to the device.

The direction rides in the least-significant bit of the address byte: `0` for a write,
`1` for a read.

- `Write` - The controller writes to the device. R/W bit `0`.
- `Read` - The controller reads from the device. R/W bit `1`.

### `Direction::rw_bit`

Returns the R/W bit this direction places in the low bit of the address byte.

**Returns**

`0` for [`Write`](Direction::Write), `1` for [`Read`](Direction::Read).

```rust
fn rw_bit(self) -> u8
```

## struct `Address`

An I2C device address, 7-bit or 10-bit, validated to its range.

I2C addresses come in two widths. The original 7-bit address shares its byte with the
R/W bit, so it lands on the wire as `(address << 1) | r/w`. The later 10-bit extension
stays backward compatible by spending the reserved `11110xx` prefix: the first byte is
`11110`, then the top two address bits, then the R/W bit, and the second byte is the
low eight address bits. Construct an address with [`seven_bit`](Address::seven_bit) or
[`ten_bit`](Address::ten_bit), which reject out-of-range values, then turn it into the
bytes a controller sends with [`write_frame`](Address::write_frame).

**Examples**

```
use pamoja_gpio::i2c::{Address, Direction};

// 7-bit: a BME280 at 0x76 writes as 0xEC and reads as 0xED.
let bme = Address::seven_bit(0x76)?;
let mut buf = [0u8; 2];
assert_eq!(bme.write_frame(Direction::Write, &mut buf)?, 1);
assert_eq!(buf[0], 0xEC);
assert_eq!(bme.write_frame(Direction::Read, &mut buf)?, 1);
assert_eq!(buf[0], 0xED);
```

### `Address::seven_bit`

Creates a 7-bit I2C address.

The whole range is accepted, including the addresses the specification reserves;
those are still legal on the wire (the general call address `0x00` is a broadcast,
for instance). Use [`is_reserved`](Address::is_reserved) to test for them.

**Arguments**

* `address` - the 7-bit device address, `0x00..=0x7F`.

**Returns**

The validated address.

**Errors**

[`GpioError::AddressOutOfRange`] if `address` exceeds `0x7F`.

```rust
fn seven_bit(address: u8) -> Result <Address, GpioError>
```

### `Address::ten_bit`

Creates a 10-bit I2C address.

**Arguments**

* `address` - the 10-bit device address, `0x000..=0x3FF`.

**Returns**

The validated address.

**Errors**

[`GpioError::AddressOutOfRange`] if `address` exceeds `0x3FF`.

```rust
fn ten_bit(address: u16) -> Result <Address, GpioError>
```

### `Address::value`

Returns the address value, without the R/W bit.

**Returns**

The 7- or 10-bit address as passed to the constructor.

```rust
fn value(self) -> u16
```

### `Address::is_ten_bit`

Returns `true` if this is a 10-bit address.

```rust
fn is_ten_bit(self) -> bool
```

### `Address::frame_len`

Returns the number of bytes [`write_frame`](Address::write_frame) emits.

**Returns**

`1` for a 7-bit address, `2` for a 10-bit address.

```rust
fn frame_len(self) -> usize
```

### `Address::is_reserved`

Returns `true` if a 7-bit address falls in a range the I2C specification reserves.

UM10204 reserves `0x00..=0x07` (general call and START byte, CBUS, a bus-format
code, a future code, and the Hs-mode master codes) and `0x78..=0x7F` (the 10-bit
addressing prefix and the device-ID codes), leaving `0x08..=0x77` for ordinary
devices. A 10-bit address is not reserved in this sense, so this returns `false`
for one.

**Returns**

`true` if this is a 7-bit address in `0x00..=0x07` or `0x78..=0x7F`.

```rust
fn is_reserved(self) -> bool
```

### `Address::is_general_call`

Returns `true` if this is the general call address `0x00`, the broadcast every
device on the bus listens to.

```rust
fn is_general_call(self) -> bool
```

### `Address::write_frame`

Writes the address byte(s) a controller puts on the bus for a transfer.

For a 7-bit address this is the single byte `(address << 1) | r/w`. For a 10-bit
address it is two bytes: `11110` then the top two address bits then the R/W bit,
followed by the low eight address bits. A 10-bit read in practice first addresses
the device with a write frame and then, after a repeated START, re-sends this first
byte with the read bit set; this method emits the bytes for the `direction` asked
for, leaving the START/repeated-START sequencing to the driver.

**Arguments**

* `direction` - whether the transfer reads or writes, which sets the R/W bit.
* `out` - the buffer the frame is written into; it must hold at least
  [`frame_len`](Address::frame_len) bytes.

**Returns**

The number of bytes written: `1` for a 7-bit address, `2` for a 10-bit address.

**Errors**

[`GpioError::BufferTooSmall`] if `out` is shorter than [`frame_len`](Address::frame_len).

```rust
fn write_frame(self, direction: Direction, out: &mut [u8]) -> Result <usize, GpioError>
```

