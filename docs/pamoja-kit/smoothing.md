# pamoja-kit::smoothing

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

Smoothing a noisy reading.

## struct `Smoother`

Smooths a noisy signal with an exponential moving average.

Cheap sensors are noisy, and a single bad sample should not trip an alarm or
flip an actuator. A [`Smoother`] dampens that jitter. The technique one layer
down is an exponential moving average: each output is a weighted blend of the
newest sample and the previous output, so recent readings count for more
without storing a history buffer.

**Examples**

```
use pamoja_kit::Smoother;

let mut smoother = Smoother::new(0.5);
assert_eq!(smoother.update(10.0), 10.0); // the first sample seeds the average
assert_eq!(smoother.update(0.0), 5.0); // then each output blends halfway
```

### `Smoother::new`

Creates a smoother with the given responsiveness.

**Arguments**

* `weight` - how much the newest sample counts, clamped to `[0.0, 1.0]`.
  `1.0` disables smoothing so the output follows the input; values near
  `0.0` smooth heavily and react slowly.

**Returns**

A smoother awaiting its first sample.

```rust
fn new(weight: f32) -> Self
```

### `Smoother::update`

Folds a new sample into the average and returns the smoothed value.

The first sample seeds the average and is returned unchanged.

**Arguments**

* `sample` - the latest raw reading.

**Returns**

The smoothed value after including `sample`.

```rust
fn update(&mut self, sample: f32) -> f32
```

### `Smoother::value`

Returns the current smoothed value, or `None` before the first sample.

**Returns**

`Some(value)` once at least one sample has been seen, otherwise `None`.

```rust
fn value(&self) -> Option <f32>
```

### `Smoother::reset`

Forgets the smoothed value so the next sample seeds the average afresh.

```rust
fn reset(&mut self)
```

