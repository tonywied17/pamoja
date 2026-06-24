# pamoja-mesh::crc

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

CRC-16/CCITT-FALSE, the integrity check the mesh frame carries.

## struct `Crc16`

An incremental CRC-16/CCITT-FALSE accumulator.

CCITT-FALSE is the long-standing checksum of short radio frames: polynomial `0x1021`,
initial value `0xFFFF`, no reflection, and no final inversion. The accumulator lets a
checksum span more than one slice, which the mesh frame needs because it sums its
header and its payload while skipping the mutable hop-limit byte between them. For a
single contiguous slice, [`crc16`] is the one-shot form.

**Examples**

```
use pamoja_mesh::Crc16;

// Summing in two parts matches summing the whole in one go.
let mut crc = Crc16::new();
crc.update(b"1234");
crc.update(b"56789");
assert_eq!(crc.finish(), 0x29B1);
```

### `Crc16::new`

Creates an accumulator primed with the CCITT-FALSE initial value.

**Returns**

A fresh accumulator, ready for [`update`](Crc16::update).

```rust
const fn new() -> Self
```

### `Crc16::update`

Folds a slice of bytes into the running checksum.

**Arguments**

* `data` - the bytes to add to the checksum.

```rust
fn update(&mut self, data: &[u8])
```

### `Crc16::finish`

Returns the checksum of everything folded in so far.

**Returns**

The 16-bit CRC.

```rust
fn finish(&self) -> u16
```

## fn `crc16`

Computes the CRC-16/CCITT-FALSE of a single byte slice.

**Arguments**

* `data` - the bytes to check.

**Returns**

The 16-bit CRC.

**Examples**

```
use pamoja_mesh::crc16;

// The standard CRC-16/CCITT-FALSE check value over the ASCII digits "123456789".
assert_eq!(crc16(b"123456789"), 0x29B1);
```

```rust
fn crc16(data: &[u8]) -> u16
```

