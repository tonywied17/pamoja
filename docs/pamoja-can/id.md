# pamoja-can::id

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

The CAN identifier: a standard 11-bit or extended 29-bit arbitration ID.

## struct `CanId`

A CAN arbitration identifier.

CAN comes in two identifier widths: the original standard 11-bit form and the extended
29-bit form that higher-layer protocols such as J1939 use to pack a priority, a
parameter group, and addresses into the ID itself. This type holds either, always
masked to its width.

**Examples**

```
use pamoja_can::CanId;

let std = CanId::standard(0x123);
assert!(!std.is_extended());
assert_eq!(std.raw(), 0x123);

// Values wider than the identifier are masked to fit.
assert_eq!(CanId::standard(0xFFFF).raw(), 0x7FF);
```

### `CanId::standard`

Creates a standard 11-bit identifier, masking the value to fit.

**Arguments**

* `raw` - the identifier value; bits above the low 11 are dropped.

**Returns**

The identifier.

```rust
fn standard(raw: u16) -> CanId
```

### `CanId::extended`

Creates an extended 29-bit identifier, masking the value to fit.

**Arguments**

* `raw` - the identifier value; bits above the low 29 are dropped.

**Returns**

The identifier.

```rust
fn extended(raw: u32) -> CanId
```

### `CanId::raw`

Returns the identifier value.

**Returns**

The raw bits, already masked to the identifier's width.

```rust
fn raw(&self) -> u32
```

### `CanId::is_extended`

Reports whether this is an extended 29-bit identifier.

**Returns**

`true` for an extended identifier, `false` for a standard one.

```rust
fn is_extended(&self) -> bool
```

