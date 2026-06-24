# pamoja-kit::complementary

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

Fusing a fast rate sensor with a slow absolute one.

## struct `Complementary`

Blends a drifting rate measurement with a noisy absolute one into a steady estimate.

A gyroscope gives a smooth rate of turn but drifts over time; an accelerometer gives an
absolute tilt that is right on average but noisy. A complementary filter trusts the rate
over the short term and the absolute reading over the long term, so the result is both
smooth and drift-free. Each step it integrates the rate onto its estimate, then nudges
that toward the absolute reading by an amount set by `alpha`. The same filter fuses any
fast-rate-plus-slow-absolute pair, not only an IMU.

**Examples**

```
use pamoja_kit::Complementary;

// Heavily trust the integrated rate, lightly correct toward the absolute reading.
let mut tilt = Complementary::new(0.98, 0.0);
// The rate reads +10 per second for 0.1 s while the absolute reads about 1.
let angle = tilt.update(10.0, 1.0, 0.1);
assert!((angle - 1.0).abs() < 0.05); // about 0.98 * 1 + 0.02 * 1
```

### `Complementary::new`

Creates a filter.

**Arguments**

* `alpha` - the weight on the integrated rate, in `[0.0, 1.0]`; near `1.0` trusts the
  rate and corrects slowly, near `0.0` follows the absolute reading. Clamped to the
  unit interval.
* `initial` - the starting estimate.

**Returns**

A filter seeded with `initial`.

```rust
fn new(alpha: f32, initial: f32) -> Self
```

### `Complementary::update`

Fuses a rate and an absolute reading over a time step and returns the new estimate.

**Arguments**

* `rate` - the rate of change, such as degrees per second from a gyroscope.
* `absolute` - the absolute reading, such as a tilt from an accelerometer.
* `dt` - the time since the previous update.

**Returns**

The fused estimate, `alpha * (estimate + rate * dt) + (1 - alpha) * absolute`.

```rust
fn update(&mut self, rate: f32, absolute: f32, dt: f32) -> f32
```

### `Complementary::estimate`

Returns the current fused estimate.

```rust
fn estimate(&self) -> f32
```

