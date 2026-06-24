# pamoja-can::error

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

The error type for CAN framing.

## enum `CanError`

What can go wrong building a CAN frame.

- `DataTooLong` - The data is longer than the frame kind allows (8 bytes for classic CAN, 64 for CAN-FD).
- `InvalidFdLength` - The data length is not one a CAN-FD frame can carry. CAN-FD allows 0 to 8 bytes, then only 12, 16, 20, 24, 32, 48, and 64.

