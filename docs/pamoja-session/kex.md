# pamoja-session::kex

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

X25519 key agreement (RFC 7748) and HKDF-SHA256 (RFC 5869): how two devices that
already hold each other's public key arrive at the same session key without ever
sending it.

The raw X25519 shared secret is never used as a key directly. It is run through
HKDF-SHA256, salted with a fresh per-session value and bound to both public keys,
so each session gets an independent key and the key is tied to the specific pair
of devices. The tests pin both primitives to their RFC reference vectors.

## struct `AgreementKey`

A device's long-term key-agreement secret.

This is the private half a device uses to agree a session key with a peer. It is
built from a 32-byte seed the device is provisioned with and keeps in secure
storage, so the same agreement key is recreated deterministically across reboots.

It is separate from the device's ed25519 signing identity in `pamoja-security`.
Key agreement gives confidentiality; it does not by itself prove who the peer is.
A deployment authenticates the peer by pinning its [`public`](AgreementKey::public)
value, or by signing that value with the peer's `pamoja-security` identity, the
same way it already pins a signing identity. Without that pinning the channel is
private but unauthenticated and a man in the middle is possible.

**Examples**

```
use pamoja_session::AgreementKey;

let device = AgreementKey::from_seed(&[7u8; 32]);
let public = device.public();
// `public.to_bytes()` is what a peer pins or has signed to trust this device.
assert_eq!(public.to_bytes().len(), 32);
```

### `AgreementKey::from_seed`

Builds a key-agreement secret from a 32-byte seed.

**Arguments**

* `seed` - the 32 secret bytes the key is derived from.

**Returns**

The agreement key.

```rust
fn from_seed(seed: &[u8 ; 32]) -> Self
```

### `AgreementKey::public`

Returns the public key a peer needs to agree a session with this device.

**Returns**

The matching [`AgreementPublicKey`], safe to share once it is authenticated.

```rust
fn public(&self) -> AgreementPublicKey
```

## struct `AgreementPublicKey`

The public half of a device's key-agreement key.

A device holds the authenticated public keys of the peers it will talk to and
uses them to agree a session key. It is 32 bytes on the wire.

### `AgreementPublicKey::from_bytes`

Reconstructs a public key from its 32-byte form.

**Arguments**

* `bytes` - the 32-byte encoded public key.

**Returns**

The public key. Every 32-byte value is a syntactically valid X25519 public
key, so this cannot fail; authenticating that the key belongs to the expected
device is the caller's responsibility.

```rust
fn from_bytes(bytes: &[u8 ; 32]) -> Self
```

### `AgreementPublicKey::to_bytes`

Returns the 32-byte wire form of this public key.

**Returns**

The public key encoded as 32 bytes.

```rust
fn to_bytes(&self) -> [u8 ; 32]
```

