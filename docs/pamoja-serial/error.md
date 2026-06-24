# pamoja-serial::error

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

The error type for serial-line framing.

## enum `SerialError`

What can go wrong framing or unframing a serial packet.

- `BufferTooSmall` - The caller's output buffer is too small to hold the encoded frame or the decoded payload. The `max_encoded_len` helpers size a buffer that always fits.
- `InvalidEscape` - A SLIP escape byte was followed by a byte that is neither the escaped-delimiter nor the escaped-escape marker, so the frame is corrupt.
- `TruncatedFrame` - A frame ended early: a SLIP frame stopped in the middle of an escape sequence, or a COBS code byte claimed more data than the frame actually carried.

