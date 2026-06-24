# pamoja-zenoh::keyexpr

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

The Zenoh key-expression language: validity, canonical form, and matching.

A key expression is a `/`-joined list of non-empty chunks. A chunk is either a literal, the
single-chunk wildcard `*` (one non-empty chunk), the multi-chunk wildcard `**` (zero or more
chunks), or a literal carrying the sub-chunk wildcard `$*` (any run of characters, including
none, within one chunk). A concrete key carries no wildcards. Leading, trailing, and doubled
`/` are forbidden, as are the bare characters `*`, `$`, `?`, and `#` outside the wildcard forms.

The rules follow the Zenoh key-expression specification, including its canonical-form rules:
`**/**` collapses to `**`, `**/*` reorders to `*/**`, `$*$*` collapses to `$*`, and a chunk that
is exactly `$*` becomes `*`.

## fn `is_valid`

Returns whether a string is a well-formed key expression.

**Arguments**

* `ke` - the candidate key expression.

**Returns**

`true` if `ke` is `/`-joined non-empty chunks with no leading, trailing, or doubled `/`, where
each chunk is `*`, `**`, or a literal in which `*` appears only as part of `$*` and `$` only
before `*`, with no `?` or `#`.

```rust
fn is_valid(ke: &str) -> bool
```

## fn `is_canon`

Returns whether a key expression is valid and in canonical form.

**Arguments**

* `ke` - the candidate key expression.

**Returns**

`true` if `ke` equals its own [`canonize`] output, so two expressions selecting the same keys
compare equal as strings.

```rust
fn is_canon(ke: &str) -> bool
```

## fn `canonize`

Returns the canonical form of a key expression, or `None` if it is invalid.

**Arguments**

* `ke` - the key expression to canonicalize.

**Returns**

`Some(canonical)` for a valid `ke`, applying the canonical-form rules (`**/**` to `**`, `**/*`
to `*/**`, `$*$*` to `$*`, and a `$*` chunk to `*`); `None` if `ke` is not a valid key
expression.

**Examples**

```
use pamoja_zenoh::keyexpr::canonize;

assert_eq!(canonize("robot/sensor/**/*").as_deref(), Some("robot/sensor/*/**"));
assert_eq!(canonize("a/**/**/b").as_deref(), Some("a/**/b"));
assert_eq!(canonize("a//b"), None); // a doubled slash is not a valid key expression
```

```rust
fn canonize(ke: &str) -> Option <String>
```

## fn `matches`

Returns whether a concrete key is selected by a pattern key expression.

**Arguments**

* `pattern` - the key expression to test against; it may contain wildcards.
* `key` - the concrete key being routed; it must be valid and carry no wildcards.

**Returns**

`true` if `key` is one of the keys `pattern` selects. Returns `false` if `pattern` is not a
valid key expression, or if `key` is not a valid concrete key.

**Examples**

```
use pamoja_zenoh::keyexpr::matches;

assert!(matches("room275/*/temperature", "room275/device1/temperature"));
assert!(!matches("room275/*/temperature", "room275/temperature")); // `*` needs one chunk
assert!(matches("organizationA/**/temperature", "organizationA/temperature")); // `**` allows none
assert!(matches("thermometer$*/temperature", "thermometer1/temperature"));
```

```rust
fn matches(pattern: &str, key: &str) -> bool
```

