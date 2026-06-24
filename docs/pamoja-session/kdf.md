# pamoja-session::kdf

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

Keyed hashing primitives: HMAC-SHA256 (RFC 2104, FIPS 198-1) and HKDF-SHA256
(RFC 5869).

These are the building blocks the session key agreement uses, exposed so a host can
reuse the same audited, vector-pinned primitives instead of pulling in a second
crypto stack. A local-first dashboard, for example, derives a per-session key from a
pairing secret with [`hkdf_sha256`] and authenticates each command with
[`hmac_sha256`].

## fn `hmac_sha256`

Computes HMAC-SHA256 over a message with a key of any length.

**Arguments**

* `key` - the secret key; any length is accepted, as HMAC defines.
* `message` - the bytes to authenticate.

**Returns**

The 32-byte message authentication code.

**Examples**

```
// RFC 4231 test case 2.
let mac = pamoja_session::hmac_sha256(b"Jefe", b"what do ya want for nothing?");
assert_eq!(mac[..4], [0x5b, 0xdc, 0xc1, 0x46]);
```

```rust
fn hmac_sha256(key: &[u8], message: &[u8]) -> [u8 ; 32]
```

## fn `hkdf_sha256`

Derives output key material from input keying material with HKDF-SHA256.

Extracts a pseudorandom key from `salt` and `ikm`, then expands it under `info` to
fill `out`.

**Arguments**

* `salt` - a non-secret salt; a fresh per-session value gives each session its own key.
* `ikm` - the input keying material, such as a shared or pairing secret.
* `info` - a context label binding the output to its purpose.
* `out` - the buffer to fill with derived key material.

**Panics**

Panics if `out` is longer than HKDF-SHA256's `255 * 32`-byte limit.

```rust
fn hkdf_sha256(salt: &[u8], ikm: &[u8], info: &[u8], out: &mut [u8])
```

