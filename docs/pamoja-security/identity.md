# pamoja-security::identity

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

Device identities: the private key that signs and the public key that verifies.

## struct `DeviceIdentity`

A device's private signing identity.

This is the secret half of a device's identity: the key it uses to sign its own
telemetry so a gateway or auditor can later prove the data came from this device
and was not tampered with. It is built from a 32-byte seed, which a device is
provisioned with and keeps in secure storage, so the same identity is recreated
deterministically across reboots without generating a new key each time.

Signing is deterministic and needs no randomness, so this works unchanged on a
microcontroller.

**Examples**

```
use pamoja_security::DeviceIdentity;

let device = DeviceIdentity::from_seed(&[7u8; 32]);
let signature = device.sign(b"fridge-1: 4.8C");
assert!(device.public().verify(b"fridge-1: 4.8C", &signature).is_ok());
```

### `DeviceIdentity::from_seed`

Builds an identity from a 32-byte secret seed.

**Arguments**

* `seed` - the 32 secret bytes the identity is derived from.

**Returns**

The device identity.

```rust
fn from_seed(seed: &[u8 ; 32]) -> Self
```

### `DeviceIdentity::public`

Returns the public identity others use to verify this device's signatures.

**Returns**

The matching [`PublicIdentity`].

```rust
fn public(&self) -> PublicIdentity
```

### `DeviceIdentity::sign`

Signs a payload with this device's key.

**Arguments**

* `payload` - the bytes to sign, such as an encoded reading.

**Returns**

A [`Signature`] over `payload`.

```rust
fn sign(&self, payload: &[u8]) -> Signature
```

## struct `PublicIdentity`

A device's public identity: it names the device and verifies its signatures.

This is the public half of a device's identity, safe to share and distribute. A
gateway holds the public identities of the devices it trusts and uses them to
check that each signed payload is authentic and unaltered.

### `PublicIdentity::from_bytes`

Reconstructs a public identity from its 32-byte form.

**Arguments**

* `bytes` - the 32-byte encoded public key.

**Returns**

The public identity.

**Errors**

Returns [`Error::Auth`](pamoja_core::Error::Auth) if `bytes` is not a valid
public key.

```rust
fn from_bytes(bytes: &[u8 ; 32]) -> Result <Self>
```

### `PublicIdentity::to_bytes`

Returns the 32-byte wire form of this identity.

**Returns**

The public key encoded as 32 bytes.

```rust
fn to_bytes(&self) -> [u8 ; 32]
```

### `PublicIdentity::fingerprint`

Returns a short hex fingerprint of this identity for logs and displays.

The fingerprint is the first eight bytes of the public key in hex. It is a
convenient label, not a substitute for the full key when checking trust.

**Returns**

A 16-character lowercase hex string.

```rust
fn fingerprint(&self) -> String
```

### `PublicIdentity::verify`

Verifies that `signature` covers `payload` and was made by this identity.

**Arguments**

* `payload` - the bytes the signature is expected to cover.
* `signature` - the signature to check.

**Returns**

`Ok(())` if the signature is authentic for `payload`.

**Errors**

Returns [`Error::Auth`](pamoja_core::Error::Auth) if the signature does not
match, which means the payload was altered or was not signed by this device.

```rust
fn verify(&self, payload: &[u8], signature: &Signature) -> Result <()>
```

