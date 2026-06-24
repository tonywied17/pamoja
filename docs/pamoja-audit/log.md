# pamoja-audit::log

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

Building and verifying a signed, hash-chained audit log.

## struct `AuditLog`

Appends signed, hash-chained entries to a tamper-evident log.

Each [`append`](AuditLog::append) signs the new entry's digest and links it to
the previous entry, so the log can later be proven complete and unaltered. The
log holds the signing identity and the chain head; it does not store the entries
itself, so the caller persists each entry's [`to_bytes`](Entry::to_bytes) to
durable storage (a file or SD card in the field) and rebuilds the chain from
there.

**Examples**

```
use pamoja_audit::{verify_chain, AuditLog};
use pamoja_security::DeviceIdentity;

let device = DeviceIdentity::from_seed(&[9u8; 32]);
let public = device.public();

let mut log = AuditLog::new(device);
let entries = [log.append(b"4.6C"), log.append(b"4.9C")];

assert!(verify_chain(&public, &entries).is_ok());
```

### `AuditLog::new`

Starts a fresh log signed by `identity`.

**Arguments**

* `identity` - the device identity that signs each entry.

**Returns**

An empty log positioned at the first entry.

```rust
fn new(identity: DeviceIdentity) -> Self
```

### `AuditLog::resume`

Resumes a log after its last entry, to keep appending across a restart.

**Arguments**

* `identity` - the device identity that signs each entry.
* `last` - the most recent entry already in durable storage.

**Returns**

A log positioned to append after `last`.

```rust
fn resume(identity: DeviceIdentity, last: &Entry) -> Self
```

### `AuditLog::append`

Appends `payload`, returning the new signed, chained entry.

The caller persists the returned entry's [`to_bytes`](Entry::to_bytes).

**Arguments**

* `payload` - the bytes to record, such as an encoded reading.

**Returns**

The new [`Entry`].

```rust
fn append(&mut self, payload: &[u8]) -> Entry
```

## struct `Verifier`

Verifies a log's entries in sequence against the signer's public identity.

A verifier checks each entry's index, its link to the previous entry, and its
signature, advancing only on success. Feed it entries oldest first; the first
failure is the point the log was tampered with.

### `Verifier::new`

Creates a verifier for a log signed by `public`, starting at the first entry.

**Arguments**

* `public` - the public identity expected to have signed the log.

**Returns**

A verifier positioned at the first entry.

```rust
fn new(public: PublicIdentity) -> Self
```

### `Verifier::check`

Verifies the next entry in sequence, advancing the verifier on success.

**Arguments**

* `entry` - the next entry in the log.

**Returns**

`Ok(())` if the entry is in sequence, correctly chained, and authentically
signed.

**Errors**

Returns [`Error::Auth`](pamoja_core::Error::Auth) if the entry is out of
sequence, its chain link is wrong, or its signature does not verify.

```rust
fn check(&mut self, entry: &Entry) -> Result <()>
```

## fn `verify_chain`

Verifies a whole chain of entries from the start against `public`.

**Arguments**

* `public` - the public identity expected to have signed the log.
* `entries` - the log's entries, oldest first.

**Returns**

`Ok(())` if every entry is in sequence, correctly chained, and authentic.

**Errors**

Returns [`Error::Auth`](pamoja_core::Error::Auth) at the first entry that is out
of sequence, broken in the chain, or not authentically signed.

```rust
fn verify_chain(public: &PublicIdentity, entries: &[Entry]) -> Result <()>
```

