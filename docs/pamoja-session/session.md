# pamoja-session::session

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

Sessions: an ordered, replay-protected channel of authenticated-encrypted
messages between two devices that have agreed a key.

## enum `Role`

Which end of a session a device is.

Both ends derive the same key, but each tags its outgoing messages with a
different direction byte in the nonce, so the two directions never share a
nonce under the one key and a message a device sends can never be opened as one
it expected to receive.

- `Initiator` - The device that opens the session. Its public key is ordered first when the key is derived, and it tags the messages it sends as the initiator direction.
- `Responder` - The device that answers. Its public key is ordered second, and it tags the messages it sends as the responder direction.

## struct `Sealed`

The out-of-band header of a sealed message: the counter that orders it and the
tag that authenticates it.

Both values travel alongside the ciphertext to the peer. The peer needs the
counter to rebuild the nonce and to reject replays, and the tag to verify the
message was not altered.

Fields:

- `counter: u64` - The monotonically increasing counter naming this message within the session.
- `tag: [u8 ; 16]` - The 16-byte ChaCha20-Poly1305 tag over the ciphertext and its associated data.

## struct `Session`

A confidential, tamper-evident, replay-protected channel with one peer.

A session holds the agreed key, the counter for the messages this device sends,
and a sliding window of the counters it has accepted from the peer. Sealing a
message encrypts it and stamps it with the next counter; opening one verifies it
and rejects anything that fails authentication or repeats a counter.

A session is deliberately not `Clone`: two copies would reuse counters and so
reuse nonces, which breaks the AEAD's guarantees. Establish a fresh session
instead.

**Examples**

```
use pamoja_session::{AgreementKey, Role, Session};

// Each device is provisioned with its own seed and knows the other's public key.
let fridge = AgreementKey::from_seed(&[1u8; 32]);
let gateway = AgreementKey::from_seed(&[2u8; 32]);

// A fresh salt is agreed in the clear at the start of each session.
let salt = [9u8; 16];
let mut device = Session::establish(&fridge, &gateway.public(), &salt, Role::Initiator);
let mut peer = Session::establish(&gateway, &fridge.public(), &salt, Role::Responder);

// The device seals a reading; the ciphertext, counter, and tag go on the wire.
let mut message = *b"4.8C";
let sealed = device.seal(&mut message, b"fridge-1");

// The gateway opens it, recovering the reading and proving it is authentic.
peer.open(&sealed, &mut message, b"fridge-1").expect("authentic message");
assert_eq!(&message, b"4.8C");
```

### `Session::establish`

Establishes a session with a peer from this device's agreement key and the
peer's authenticated public key.

Both devices call this with the same `salt` and opposite [`Role`]s and arrive
at the same key. The salt is a fresh per-session value the two sides exchange
in the clear before sealing anything; reusing a salt with the same pair of
keys reuses the session key, so it must change each session (a counter kept in
power-loss-safe storage, or a nonce from a handshake, both work).

**Arguments**

* `local` - this device's key-agreement secret.
* `peer` - the peer's public key, already authenticated by pinning or signature.
* `salt` - the fresh per-session salt both sides share.
* `role` - whether this device is the [`Role::Initiator`] or [`Role::Responder`].

**Returns**

A session ready to seal and open messages with the peer.

```rust
fn establish(local: &AgreementKey, peer: &AgreementPublicKey, salt: &[u8], role: Role,) -> Self
```

### `Session::seal`

Seals a message for the peer, encrypting `buf` in place and stamping it with
the next counter.

The associated data `aad` is authenticated but not encrypted, so it is
readable on the wire yet cannot be altered: a device identifier or a routing
header belongs here. After this returns, `buf` holds the ciphertext and the
returned [`Sealed`] holds the counter and tag to send with it.

**Arguments**

* `buf` - the plaintext, replaced in place by the ciphertext of equal length.
* `aad` - associated data to authenticate alongside the message.

**Returns**

The [`Sealed`] header (counter and tag) for this message.

```rust
fn seal(&mut self, buf: &mut [u8], aad: &[u8]) -> Sealed
```

### `Session::open`

Opens a message from the peer, verifying it and decrypting `buf` in place.

The message is rejected if its counter has already been seen or is older than
the replay window still tracks, and if its tag does not authenticate. On any
rejection `buf` is left zeroed, so a failed open never yields readable bytes.
The replay window only advances on a message that authenticates, so a forged
counter cannot push genuine messages out of the window.

**Arguments**

* `sealed` - the counter and tag that arrived with the ciphertext.
* `buf` - the ciphertext, replaced in place by the plaintext on success.
* `aad` - the same associated data the sender authenticated.

**Returns**

`Ok(())` if the message is authentic and fresh, with `buf` now the plaintext.

**Errors**

Returns [`SessionError::Replayed`] if the counter repeats or is too old, or
[`SessionError::Inauthentic`] if the message fails authentication.

```rust
fn open(&mut self, sealed: &Sealed, buf: &mut [u8], aad: &[u8],) -> Result <(), SessionError>
```

