# pamoja-kit::kalman

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

Getting a steady value from a jittery sensor.

## struct `Kalman`

Estimates a steady value from noisy readings with a one-dimensional Kalman filter.

Where [`Smoother`](crate::Smoother) blends with a fixed weight, a Kalman filter sets the
blend from how much it trusts its estimate versus each reading, so it settles quickly and
then holds steady. It tracks an estimate and its uncertainty: each step it grows the
uncertainty by the process noise (how much the true value may drift between readings),
then pulls the estimate toward the new reading by a gain derived from that uncertainty
and the measurement noise (how noisy the sensor is). It suits a slowly changing quantity
read by a noisy sensor: a battery voltage, a tank level, a temperature.

**Examples**

```
use pamoja_kit::Kalman;

// Low process noise, higher measurement noise: trust history, smooth hard.
let mut level = Kalman::new(0.01, 1.0, 0.0);
let mut value = 0.0;
for reading in [10.0, 9.0, 11.0, 10.0, 10.0] {
    value = level.update(reading);
}
assert!((value - 10.0).abs() < 1.0); // settles near the true 10
```

### `Kalman::new`

Creates a filter.

**Arguments**

* `process_noise` - how much the true value may change between readings; larger
  tracks faster, smaller smooths harder. Its magnitude is used.
* `measurement_noise` - how noisy each reading is; larger trusts readings less. Its
  magnitude is used.
* `initial` - the starting estimate, used until the first reading replaces it.

**Returns**

A filter awaiting its first reading.

```rust
fn new(process_noise: f32, measurement_noise: f32, initial: f32) -> Self
```

### `Kalman::update`

Folds in a reading and returns the updated estimate.

The first reading seeds the estimate and is returned unchanged.

**Arguments**

* `reading` - the latest measurement.

**Returns**

The filtered estimate after this reading.

```rust
fn update(&mut self, reading: f32) -> f32
```

### `Kalman::estimate`

Returns the current estimate.

```rust
fn estimate(&self) -> f32
```

