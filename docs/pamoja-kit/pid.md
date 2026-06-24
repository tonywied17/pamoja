# pamoja-kit::pid

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

Holding a value at a target with a PID controller.

## struct `Pid`

Drives a measured value to a target by blending proportional, integral, and derivative
terms.

This is the workhorse continuous controller behind "keep it here": hold a heater at a
temperature, a pump at a pressure, a motor at a speed. It sums three responses to the
error (target minus measurement): the proportional term reacts to the error now, the
integral term removes the steady offset the proportional term leaves behind, and the
derivative term damps overshoot by reacting to how fast the error is changing. The gains
`kp`, `ki`, and `kd` weight them. The output is clamped to a configurable range, and the
integral is held back from winding up past that range while the output is saturated, the
standard clamping anti-windup.

For the simplest on/off case (a fridge, a tank pump) reach for
[`Thermostat`](crate::Thermostat) instead; a PID is for a smooth, proportional actuator.

**Examples**

```
use pamoja_kit::Pid;

// Proportional-only: the command is the gain times the error.
let mut pid = Pid::new(2.0, 0.0, 0.0);
assert_eq!(pid.update(10.0, 7.0, 1.0), 6.0); // error 3 times kp 2
```

### `Pid::new`

Creates a PID controller with the given gains and no output limit.

**Arguments**

* `kp` - proportional gain.
* `ki` - integral gain.
* `kd` - derivative gain.

**Returns**

A controller with a cleared history and unbounded output.

```rust
fn new(kp: f32, ki: f32, kd: f32) -> Self
```

### `Pid::with_limits`

Limits the output to `[min, max]`, also bounding the integral so it cannot wind up
beyond the range while the output is saturated.

**Arguments**

* `min` - the lowest output.
* `max` - the highest output. If `max` is below `min` the two are swapped.

**Returns**

The controller, for chaining after [`new`](Pid::new).

```rust
fn with_limits(mut self, min: f32, max: f32) -> Self
```

### `Pid::update`

Computes the control output for one time step.

**Arguments**

* `setpoint` - the target value.
* `measurement` - the latest measured value.
* `dt` - the time since the previous update, in the unit `ki` and `kd` assume. A
  value at or below zero skips the integral and derivative updates.

**Returns**

The control output, clamped to the configured limits.

```rust
fn update(&mut self, setpoint: f32, measurement: f32, dt: f32) -> f32
```

### `Pid::reset`

Clears the integral and derivative history.

```rust
fn reset(&mut self)
```

