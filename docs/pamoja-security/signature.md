# pamoja-security::signature

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

The signature a device produces over a payload.

## struct `Signature`

A detached ed25519 signature over a payload.

A signature is 64 bytes on the wire. Send it alongside the payload it covers, and
the receiver checks it with the signer's [`PublicIdentity`](crate::PublicIdentity)
to confirm the payload came from that device and was not altered in transit.

### `Signature::to_bytes`

Returns the 64-byte wire form of the signature.

**Returns**

The signature encoded as 64 bytes.

```rust
fn to_bytes(&self) -> [u8 ; 64]
```

### `Signature::from_bytes`

Reconstructs a signature from its 64-byte wire form.

The bytes are not validated here; an invalid signature is rejected when it is
checked by [`PublicIdentity::verify`](crate::PublicIdentity::verify).

**Arguments**

* `bytes` - the 64-byte encoded signature.

**Returns**

The signature.

```rust
fn from_bytes(bytes: &[u8 ; 64]) -> Self
```

