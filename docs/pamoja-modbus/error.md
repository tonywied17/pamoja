# pamoja-modbus::error

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

The error type for Modbus framing.

## enum `ModbusError`

What can go wrong building or reading a Modbus RTU frame.

- `FrameTooShort` - A frame is shorter than the smallest valid RTU ADU (address, function, CRC).
- `FrameTooLong` - A frame or PDU is longer than the 256-byte RTU maximum allows.
- `CrcMismatch` - A received frame's CRC does not match its contents, so the frame is corrupt.
- `InvalidValueCount` - A write request named a number of values a single request cannot carry (it must be between one and the function's maximum).
- `MalformedResponse` - A response PDU is truncated or its declared byte count does not match its data.

