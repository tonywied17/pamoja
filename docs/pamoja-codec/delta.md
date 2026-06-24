# pamoja-codec::delta

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

Compact batch encoding for metered links: delta plus variable-length integers.

On a long-range radio or a metered cellular link, every byte costs power or money,
so it pays to send a batch of readings in as few bytes as possible rather than one
full-width value at a time. The functions here encode a sequence of integers as a
starting value followed by the differences between consecutive values, each
written as a variable-length integer. A slowly changing signal - a temperature, a
tank level, a battery voltage - then costs about one byte per sample instead of
eight, with no loss. [`Quantizer`] extends this to `f32` readings by rounding each
to a fixed precision first.

## fn `encode_deltas`

Encodes a batch of integer samples as a starting value plus variable-length deltas.

**Arguments**

* `samples` - the integers to encode, in order.

**Returns**

The compact encoding. A slowly changing series is far smaller than the eight bytes
per sample a raw encoding would use.

**Examples**

```
use pamoja_codec::{decode_deltas, encode_deltas};

let samples = [1000, 1001, 1003, 1002];
let bytes = encode_deltas(&samples);
assert!(bytes.len() < samples.len() * 8); // far smaller than eight bytes each
assert_eq!(decode_deltas(&bytes).unwrap(), samples);
```

```rust
fn encode_deltas(samples: &[i64]) -> Vec <u8>
```

## fn `decode_deltas`

Decodes a batch encoded by [`encode_deltas`].

**Arguments**

* `bytes` - the encoded batch.

**Returns**

The decoded samples, in order.

**Errors**

Returns [`Error::Codec`](pamoja_core::Error::Codec) if `bytes` ends in the middle
of a value or encodes an over-long integer.

```rust
fn decode_deltas(bytes: &[u8]) -> Result <Vec <i64>>
```

## struct `Quantizer`

Packs a batch of `f32` readings into a compact byte form for a metered link.

A quantizer rounds each reading to a fixed precision - set by the `scale`, where
`100.0` keeps two decimal places - turns it into an integer, and delta-encodes the
batch with [`encode_deltas`]. This is lossy by exactly the rounding step, which is
the right trade for a cheap sensor on an expensive link: a fridge temperature to
the nearest hundredth of a degree costs a byte or two per sample instead of four.
The same `scale` must be used to encode and decode.

**Examples**

```
use pamoja_codec::Quantizer;

// Quantize to 0.1 precision and pack a slowly-rising series.
let quantizer = Quantizer::new(10.0);
let readings = [20.0, 20.1, 20.2, 20.3];
let packed = quantizer.encode(&readings);
assert!(packed.len() < readings.len() * 4); // smaller than four bytes per reading

let restored = quantizer.decode(&packed).unwrap();
assert!((restored[2] - 20.2).abs() < 0.05);
```

### `Quantizer::new`

Creates a quantizer with the given precision scale.

**Arguments**

* `scale` - the multiplier applied before rounding; `100.0` keeps two decimal
  places. Must be positive.

**Returns**

The quantizer.

```rust
fn new(scale: f32) -> Self
```

### `Quantizer::encode`

Quantizes and delta-encodes a batch of readings.

**Arguments**

* `readings` - the readings to pack, in order.

**Returns**

The compact encoding of the batch.

```rust
fn encode(&self, readings: &[f32]) -> Vec <u8>
```

### `Quantizer::decode`

Decodes a batch back into readings, to within the quantizer's precision.

**Arguments**

* `bytes` - the encoding produced by [`encode`](Quantizer::encode) with the
  same scale.

**Returns**

The decoded readings, in order.

**Errors**

Returns [`Error::Codec`](pamoja_core::Error::Codec) if `bytes` is malformed.

```rust
fn decode(&self, bytes: &[u8]) -> Result <Vec <f32>>
```

