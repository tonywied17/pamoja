# pamoja-mesh::error

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

The error type for mesh framing.

## enum `MeshError`

What can go wrong building or reading a mesh frame.

- `FrameTooShort` - A frame is shorter than the header and checksum a frame must at least contain.
- `FrameTooLong` - A frame is larger than [`Frame::MAX_LEN`](crate::Frame::MAX_LEN).
- `PayloadTooLong` - A payload is larger than [`Frame::MAX_PAYLOAD`](crate::Frame::MAX_PAYLOAD), so it will not fit a single frame.
- `UnsupportedVersion` - A received frame declares a protocol version this build does not understand.
- `CrcMismatch` - A received frame's checksum does not match its contents, so the frame is corrupt.

