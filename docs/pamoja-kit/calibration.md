# pamoja-kit::calibration

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

Turning a raw reading into real-world units.

## struct `Calibration`

A linear map from raw sensor counts to calibrated units.

Most cheap analog sensors report arbitrary counts - ADC steps, a raw voltage -
that mean nothing until they are converted to real units. A [`Calibration`]
applies the line `value = scale * raw + offset`. Build it from two readings
whose true values are known, then apply it to every sample.

**Examples**

```
use pamoja_kit::Calibration;

// A humidity probe reads 0.5 V at 0 % and 2.5 V at 100 %.
let humidity = Calibration::two_point(0.5, 0.0, 2.5, 100.0);
assert_eq!(humidity.apply(1.5), 50.0);
```

### `Calibration::linear`

Builds a calibration from a scale and offset directly.

**Arguments**

* `scale` - the multiplier applied to a raw reading.
* `offset` - the constant added after scaling.

**Returns**

The calibration `value = scale * raw + offset`.

```rust
fn linear(scale: f32, offset: f32) -> Self
```

### `Calibration::two_point`

Builds a calibration from two known `(raw, value)` points.

**Arguments**

* `raw_low` - a raw reading.
* `value_low` - the true value at `raw_low`.
* `raw_high` - another raw reading.
* `value_high` - the true value at `raw_high`.

**Returns**

The line through both points. If the two raw readings are equal the slope is
undefined, so the calibration falls back to the constant `value_low`.

```rust
fn two_point(raw_low: f32, value_low: f32, raw_high: f32, value_high: f32) -> Self
```

### `Calibration::apply`

Converts a raw reading into calibrated units.

**Arguments**

* `raw` - the uncalibrated sensor reading.

**Returns**

The calibrated value.

```rust
fn apply(&self, raw: f32) -> f32
```

