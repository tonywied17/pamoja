# pamoja-gpio::error

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

The error type for on-board bus addressing and pin logic.

## enum `GpioError`

What can go wrong forming an I2C address frame.

- `AddressOutOfRange` - An address is outside its range: a 7-bit address above `0x7F`, or a 10-bit address above `0x3FF`.
- `BufferTooSmall` - The caller's output buffer is too small to hold the address frame. A 7-bit address needs one byte and a 10-bit address two; [`Address::frame_len`](crate::i2c::Address::frame_len) gives the exact count.

