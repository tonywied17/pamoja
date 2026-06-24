# pamoja-session

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

Encrypted, authenticated sessions for the pamoja SDK.

[`pamoja-security`](https://docs.rs/pamoja-security) proves a payload came from a
device and was not altered. This crate adds the other half a networked link needs:
confidentiality and a fresh, ordered, replay-protected channel, so a reading is
not just trustworthy but private, and a captured message cannot be replayed to
reopen a valve or re-trigger an alarm.

Two devices that each hold the other's authenticated public key agree a session
key with [`Session::establish`] and then exchange messages with [`Session::seal`]
and [`Session::open`]. The whole exchange is built from published standards and
the tests are pinned to their reference vectors:

- X25519 key agreement, RFC 7748, so neither side ever sends the key.
- HKDF-SHA256, RFC 5869, to derive a per-session key bound to both public keys.
- ChaCha20-Poly1305, RFC 8439, to encrypt and authenticate each message; chosen
  because the cheap hardware this SDK targets rarely has AES acceleration.

Establishing a session is deterministic given the keys and salt, and every
operation works in place on caller-owned buffers, so the crate is `no_std` and
allocation-free and runs unchanged on a microcontroller. It is the secured-channel
groundwork the security pillar builds on, ahead of full transport TLS/DTLS.

# Authenticating the peer

Key agreement gives a private channel; it does not by itself say who is on the
other end. The peer's [`AgreementPublicKey`] must be authenticated out of band,
by pinning it at provisioning time or by having it signed with the peer's
`pamoja-security` identity. Without that, the channel is confidential but open to
a man in the middle.

**Examples**

```
use pamoja_session::{AgreementKey, Role, Session};

// Each device holds its own seed and the other's authenticated public key.
let sensor = AgreementKey::from_seed(&[1u8; 32]);
let gateway = AgreementKey::from_seed(&[2u8; 32]);

// A fresh salt is exchanged in the clear to start the session.
let salt = [42u8; 16];
let mut a = Session::establish(&sensor, &gateway.public(), &salt, Role::Initiator);
let mut b = Session::establish(&gateway, &sensor.public(), &salt, Role::Responder);

// Seal a reading; the device id rides along as authenticated-but-readable data.
let mut reading = *b"tank: 18%";
let sealed = a.seal(&mut reading, b"well-3");

// The gateway opens it, recovering the reading and proving it is authentic.
b.open(&sealed, &mut reading, b"well-3").expect("authentic and fresh");
assert_eq!(&reading, b"tank: 18%");

// A replay of the same message is refused.
assert!(b.open(&sealed, &mut reading.clone(), b"well-3").is_err());
```

## Modules

- [aead](aead.md)
- [error](error.md)
- [kdf](kdf.md)
- [kex](kex.md)
- [session](session.md)

