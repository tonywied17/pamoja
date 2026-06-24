# pamoja-core::error

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

The error model shared by every pamoja crate.

A single [`Error`] type keeps failure handling uniform across capabilities and
maps cleanly onto each language binding's native error idiom, such as
exceptions or rejected promises.

## enum `Error`

The error type returned by all fallible pamoja operations.

This enum is `#[non_exhaustive]`: new variants may be added in future releases
without a breaking change, so downstream `match` expressions must include a
wildcard arm.

- `Transport` - A transport-level failure while connecting, sending, or receiving.  The payload is a human-readable description provided by the transport.
- `Io` - A device or peripheral input/output operation failed.  The payload is a human-readable description of the I/O fault.
- `Codec` - A payload could not be encoded or decoded.  The payload describes the encoding or decoding fault.
- `Closed` - The operation targeted a resource that is closed or disconnected.
- `Auth` - A security check failed, such as an invalid identity or a bad signature.  The payload describes the authentication or integrity fault.
- `Unsupported` - The requested capability is not compiled into this build.  The payload names the missing capability, for example `"mqtt"`.

## type `Result`

A specialized [`core::result::Result`] whose error type is fixed to [`Error`].

