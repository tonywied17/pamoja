# pamoja-kit::ramp

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

Easing a value toward a target at a limited rate.

## struct `Ramp`

Moves a value toward a target by at most a fixed step each update.

Commanding an actuator straight to a new value can be harsh: a motor lurches, a valve
slams, a lamp jumps. A [`Ramp`] limits how fast the commanded value may change, easing it
toward the target by at most `max_step` per update and snapping to the target once it is
within a step. It is the slew-rate limiter behind a smooth start and stop.

**Examples**

```
use pamoja_kit::Ramp;

// Start at 0, move at most 2 per step, aim for 5.
let mut ramp = Ramp::new(0.0, 2.0);
assert_eq!(ramp.update(5.0), 2.0);
assert_eq!(ramp.update(5.0), 4.0);
assert_eq!(ramp.update(5.0), 5.0); // within a step: snaps to the target
```

### `Ramp::new`

Creates a ramp starting at `start` that moves at most `max_step` per update.

**Arguments**

* `start` - the initial value.
* `max_step` - the largest change allowed per update; its magnitude is used.

**Returns**

A ramp resting at `start`.

```rust
fn new(start: f32, max_step: f32) -> Self
```

### `Ramp::update`

Moves toward `target` by at most the step and returns the new value.

**Arguments**

* `target` - the value being approached.

**Returns**

The value after one limited step, equal to `target` once within a step of it.

```rust
fn update(&mut self, target: f32) -> f32
```

### `Ramp::update_capped`

Moves toward `target` by at most `max_step` this update, overriding the fixed rate.

This is the variable-rate cousin of [`update`](Ramp::update): a bounded-acceleration
limiter passes `accel * dt` as the step so the change allowed each update scales with
the time since the last one, rather than the constant step fixed at construction.

**Arguments**

* `target` - the value being approached.
* `max_step` - the largest change allowed this update; its magnitude is used.

**Returns**

The value after one limited step, equal to `target` once within `max_step` of it.

```rust
fn update_capped(&mut self, target: f32, max_step: f32) -> f32
```

### `Ramp::value`

Returns the current value.

```rust
fn value(&self) -> f32
```

### `Ramp::set`

Jumps directly to `value`, bypassing the rate limit.

**Arguments**

* `value` - the new value.

```rust
fn set(&mut self, value: f32)
```

