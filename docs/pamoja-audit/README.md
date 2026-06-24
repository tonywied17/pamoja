# pamoja-audit

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

Tamper-evident audit logs for the pamoja SDK.

The health and cold-chain deployments this SDK targets need more than authentic
readings; they need an authentic *record* of them. A vaccine fridge's
temperature history is only useful as evidence if no one can quietly edit out an
excursion, drop an inconvenient reading, or reorder the log after the fact. This
crate provides that record by chaining signed entries together:

- [`AuditLog`] - appends entries, each signed with a
  [`DeviceIdentity`](pamoja_security::DeviceIdentity) and linked by hash to the
  entry before it.
- [`Entry`] - one record: its payload, its index, the previous entry's digest,
  and the signature, with a byte form for durable storage.
- [`Verifier`] and [`verify_chain`] - replay a stored log and confirm, against the
  device's [`PublicIdentity`](pamoja_security::PublicIdentity), that every entry
  is in sequence, correctly chained, and authentically signed.

Because each entry commits to the previous one with a SHA-256 hash and an ed25519
signature, altering a payload, reordering entries, inserting a forgery, or
dropping a record all break verification at the point of tampering. The crate is
`no_std` and synchronous, so the same log can be written on a microcontroller and
audited on a server.

**Examples**

```
use pamoja_audit::{verify_chain, AuditLog, Entry};
use pamoja_security::DeviceIdentity;

let device = DeviceIdentity::from_seed(&[9u8; 32]);
let public = device.public();

// Record two readings, persisting each entry's bytes as you would to an SD card.
let mut log = AuditLog::new(device);
let mut stored: Vec<Vec<u8>> = Vec::new();
for reading in [b"4.6C".as_slice(), b"4.9C".as_slice()] {
    stored.push(log.append(reading).to_bytes());
}

// An auditor rebuilds the chain from storage and verifies it.
let entries: Vec<Entry> = stored
    .iter()
    .map(|bytes| Entry::from_bytes(bytes))
    .collect::<pamoja_core::Result<_>>()
    .unwrap();
assert!(verify_chain(&public, &entries).is_ok());
```

## Modules

- [entry](entry.md)
- [log](log.md)

