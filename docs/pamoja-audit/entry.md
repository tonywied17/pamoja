# pamoja-audit::entry

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

A single entry in a signed, hash-chained audit log.

## struct `Entry`

One entry in a tamper-evident audit log.

An entry binds a payload both to its position in the log and to the entry before
it: it carries the payload, the entry's index, the digest of the previous entry
(the chain link), and a signature over this entry's digest. Because each entry
commits to the one before, altering, reordering, inserting, or dropping any entry
breaks the chain and is caught on verification.

### `Entry::index`

Returns this entry's position in the log, counting from zero.

**Returns**

The entry index.

```rust
fn index(&self) -> u64
```

### `Entry::previous`

Returns the digest of the previous entry that this entry chains to.

**Returns**

The previous entry's digest, or all zeros for the first entry.

```rust
fn previous(&self) -> [u8 ; 32]
```

### `Entry::payload`

Returns the entry's payload, such as an encoded reading.

**Returns**

The payload bytes.

```rust
fn payload(&self) -> &[u8]
```

### `Entry::signature`

Returns the signature over this entry's digest.

**Returns**

The entry's [`Signature`].

```rust
fn signature(&self) -> &Signature
```

### `Entry::digest`

Computes this entry's digest: the hash the signature covers and the next
entry chains to.

**Returns**

The 32-byte SHA-256 digest over the index, previous digest, and payload.

```rust
fn digest(&self) -> [u8 ; 32]
```

### `Entry::to_bytes`

Encodes the entry to bytes for durable storage.

The layout is the little-endian index, the previous digest, the signature,
then the payload.

**Returns**

The encoded entry.

```rust
fn to_bytes(&self) -> Vec <u8>
```

### `Entry::from_bytes`

Decodes an entry from its stored bytes.

**Arguments**

* `bytes` - the encoded entry, as produced by [`to_bytes`](Entry::to_bytes).

**Returns**

The decoded entry.

**Errors**

Returns [`Error::Codec`](pamoja_core::Error::Codec) if `bytes` is shorter than
an entry header.

```rust
fn from_bytes(bytes: &[u8]) -> Result <Self>
```

