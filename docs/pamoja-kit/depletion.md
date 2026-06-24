# pamoja-kit::depletion

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

Warning before a falling level runs out.

## struct `Depletion`

Predicts how soon a falling level will reach a threshold.

This is the primitive behind "warn before a tank runs dry". Feed it successive
level readings and it estimates how many more samples remain before the level
reaches a low mark, so an alert can fire with time to act on it. The technique
one layer down is a linear extrapolation of the most recent rate of fall, so it
reacts to noise and pairs well with a [`Smoother`](crate::Smoother) on the input.

**Examples**

```
use pamoja_kit::Depletion;

let mut tank = Depletion::new(0.0);
assert_eq!(tank.update(10.0), None); // first reading: no rate is known yet
assert_eq!(tank.update(8.0), Some(4)); // falling 2 per sample, 4 until empty
```

### `Depletion::new`

Creates a predictor that warns as the level approaches `threshold`.

**Arguments**

* `threshold` - the low level to predict reaching, such as an empty tank.

**Returns**

A predictor awaiting its first two readings.

```rust
fn new(threshold: f32) -> Self
```

### `Depletion::update`

Records a reading and estimates the samples until the threshold is reached.

**Arguments**

* `level` - the latest measured level.

**Returns**

`Some(0)` if the level is already at or below the threshold; `Some(n)` for
the estimated number of samples until it is reached at the current rate of
fall; or `None` if the level is steady or rising, or if this is the first
reading and no rate is known yet.

```rust
fn update(&mut self, level: f32) -> Option <u32>
```

