# pamoja-lorawan::error

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

The error type for LoRaWAN framing.

## enum `LorawanError`

What can go wrong building or reading a LoRaWAN frame.

- `PayloadTooLong` - A payload (or its frame options) is too large to fit a single frame.
- `FrameTooShort` - A received frame is shorter than its fixed header and MIC require.
- `UnsupportedMType` - A received frame's message type is not one this crate decodes here.
- `MicMismatch` - A received frame's MIC does not match its contents, so it is forged or corrupt.
- `FcntMismatch` - A received frame's counter does not match the counter expected for it.
- `MalformedFrame` - A received frame is structurally invalid in some other way.

