# pamoja-modbus::crc

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

CRC-16/MODBUS, the integrity check every Modbus RTU frame carries.

## fn `crc16`

Computes the CRC-16/MODBUS of a byte slice.

This is the checksum a Modbus RTU frame ends with, and the reason a receiver can
trust a frame that arrived over a long, electrically noisy cable: the polynomial is
`0xA001` (the reflected form of `0x8005`), the initial value is `0xFFFF`, input and
output are reflected, and there is no final inversion. A frame appends the result
low byte first.

**Arguments**

* `data` - the bytes to check: the unit address through the end of the PDU, that is,
  the whole frame except the two CRC bytes themselves.

**Returns**

The 16-bit CRC.

**Examples**

```
use pamoja_modbus::crc16;

// The standard CRC-16/MODBUS check value over the ASCII digits "123456789".
assert_eq!(crc16(b"123456789"), 0x4B37);
```

```rust
fn crc16(data: &[u8]) -> u16
```

