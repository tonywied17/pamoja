# pamoja-sim::sensor

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

Fake sensors that stand in for real hardware.

## struct `SimSensor`

A fake sensor that generates a signal from a baseline, drift, and noise.

This is the workhorse for hardware-free development: it implements the core
[`Sensor`] trait, so it drops into a `Node`, a profile, or any test exactly where
a real probe would, and it produces readings that look like the field rather than
a clean constant. A reading is the baseline plus any accumulated drift plus a
bounded pseudo-random wobble, so a control loop can be exercised against a signal
that warms, sags, or jitters the way a real one does.

The noise is deterministic for a given seed - a small xorshift generator drives
it, with no `rand` dependency - so a test that uses a `SimSensor` produces the
same sequence every run and stays reproducible in CI.

**Examples**

A noisy thermometer that warms by 0.1 degrees each reading:

```
use pamoja_core::Sensor;
use pamoja_sim::SimSensor;

let mut probe = SimSensor::new(20.0).with_drift(0.1).with_noise(0.05).with_seed(7);
let first = probe.read().await?;
assert!((first - 20.0).abs() <= 0.05); // the first reading sits near the baseline
```

### `SimSensor::new`

Creates a sensor that reads `baseline` with no drift or noise.

**Arguments**

* `baseline` - the value the sensor reads before drift and noise are added.

**Returns**

A steady sensor; add [`with_drift`](SimSensor::with_drift) and
[`with_noise`](SimSensor::with_noise) to make it lifelike.

```rust
fn new(baseline: f32) -> Self
```

### `SimSensor::with_drift`

Sets how much the baseline moves each reading, modelling a slow trend.

**Arguments**

* `per_read` - the amount added to the baseline after each reading; negative
  values sag the signal downward.

**Returns**

The updated sensor, for chaining.

```rust
fn with_drift(mut self, per_read: f32) -> Self
```

### `SimSensor::with_noise`

Sets the amplitude of the bounded noise added to each reading.

**Arguments**

* `amplitude` - the largest magnitude the noise can reach; its magnitude is
  used, and each reading wobbles within plus or minus this amount.

**Returns**

The updated sensor, for chaining.

```rust
fn with_noise(mut self, amplitude: f32) -> Self
```

### `SimSensor::with_seed`

Sets the seed for the noise generator, making a run reproducible.

**Arguments**

* `seed` - the generator seed; zero is replaced with a fixed non-zero value,
  since the xorshift generator cannot start from zero.

**Returns**

The updated sensor, for chaining.

```rust
fn with_seed(mut self, seed: u32) -> Self
```

## struct `Replay`

A fake sensor that replays a fixed sequence of readings.

Where a [`SimSensor`] generates a signal, a `Replay` plays back exact values in
order, which is what a deterministic test or a scripted demo wants: spell out the
readings that tell the story, and the sensor yields them one per
[`read`](Sensor::read). A one-shot replay reports [`Error::Closed`] once the
sequence is exhausted; a repeating one loops forever.

**Examples**

```
use pamoja_core::Sensor;
use pamoja_sim::Replay;

let mut gauge = Replay::new(vec![1.0, 1.2, 1.9]);
assert_eq!(gauge.read().await?, 1.0);
assert_eq!(gauge.read().await?, 1.2);
```

### `Replay::new`

Creates a sensor that yields `readings` once, then reports closed.

**Arguments**

* `readings` - the values to play back in order.

**Returns**

A one-shot replay sensor.

```rust
fn new(readings: Vec <f32>) -> Self
```

### `Replay::repeating`

Creates a sensor that yields `readings` in a loop forever.

**Arguments**

* `readings` - the values to play back in order, repeating from the start.

**Returns**

A repeating replay sensor.

```rust
fn repeating(readings: Vec <f32>) -> Self
```

