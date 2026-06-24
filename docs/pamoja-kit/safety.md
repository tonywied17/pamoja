# pamoja-kit::safety

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

Keeping a moving robot safe: stop on command, stop on silence, and never lurch.

A robot that drives itself is dangerous when something goes wrong, so safety here is a real
feature rather than an afterthought. Three pieces compose into a [`SafetyGate`] that every
motion command passes through: an [`EStop`] that latches the machine stopped until a person
clears it, a [`Watchdog`] that stops the machine if the commands stop arriving (a crashed
controller or a dropped link), and [`Limits`] that bound how fast and how abruptly the robot
may move. The gate fails safe: when stopped it commands zero, and it eases back up through the
limits rather than jumping.

## struct `EStop`

An emergency stop that latches: once engaged it holds until explicitly reset.

Unlike a transient condition, an e-stop must stay tripped after the event that caused it, so a
person decides when it is safe to move again. While engaged, [`gate`](EStop::gate) forces any
command to a full stop.

**Examples**

```
use pamoja_kit::{EStop, Twist};

let mut estop = EStop::new();
let cmd = Twist::planar(1.0, 0.0);
assert_eq!(estop.gate(cmd), cmd); // clear: passes through

estop.engage();
assert_eq!(estop.gate(cmd), Twist::zero()); // latched: stopped
estop.reset();
assert_eq!(estop.gate(cmd), cmd); // cleared by a person
```

### `EStop::new`

Creates a cleared e-stop.

**Returns**

An e-stop that is not engaged.

```rust
fn new() -> Self
```

### `EStop::engage`

Engages the stop; it latches until [`reset`](EStop::reset).

```rust
fn engage(&mut self)
```

### `EStop::reset`

Clears the stop, allowing motion again.

```rust
fn reset(&mut self)
```

### `EStop::is_engaged`

Returns whether the stop is currently engaged.

**Returns**

`true` while latched.

```rust
fn is_engaged(&self) -> bool
```

### `EStop::gate`

Returns the command to apply: the input when clear, a full stop when engaged.

**Arguments**

* `desired` - the command that would be applied if clear.

**Returns**

`desired` when clear, or [`Twist::zero`] when engaged.

```rust
fn gate(&self, desired: Twist) -> Twist
```

## struct `Watchdog`

A deadman timer: it expires unless fed often enough, catching a stalled controller or link.

Autonomy assumes a stream of fresh commands. If that stream stops, because the controller hung
or the radio dropped, the last command must not run forever. A watchdog counts the time since it
was last fed and reports expiry once that exceeds its timeout; the caller feeds it each time a
fresh command arrives. It accumulates a supplied `dt` rather than reading a clock, so it works
the same on a microcontroller with no wall time.

**Examples**

```
use pamoja_kit::Watchdog;

let mut dog = Watchdog::new(0.5); // expire after 0.5 s of silence
dog.feed();
assert!(!dog.update(0.3)); // 0.3 s since feeding: still alive
assert!(dog.update(0.3)); // 0.6 s total: expired
dog.feed(); // a fresh command revives it
assert!(!dog.is_expired());
```

### `Watchdog::new`

Creates a watchdog that expires after `timeout` without being fed.

**Arguments**

* `timeout` - the allowed silence before expiry; its magnitude is used.

**Returns**

A freshly fed watchdog.

```rust
fn new(timeout: f32) -> Self
```

### `Watchdog::feed`

Feeds the watchdog, resetting the silence timer.

```rust
fn feed(&mut self)
```

### `Watchdog::update`

Advances the timer by `dt` and returns whether it has expired.

**Arguments**

* `dt` - the time since the previous update; its magnitude is used.

**Returns**

`true` if the watchdog is now expired.

```rust
fn update(&mut self, dt: f32) -> bool
```

### `Watchdog::is_expired`

Returns whether the watchdog is currently expired.

**Returns**

`true` if the time since feeding exceeds the timeout.

```rust
fn is_expired(&self) -> bool
```

## struct `Limits`

Bounds a robot's speed and acceleration so commands stay within what the machine can do safely.

A raw command can be too fast or too sudden: a full-speed setpoint snaps the wheels, a hard
reverse strips traction. [`Limits`] caps the planar speed and yaw rate, then eases each toward
the capped target at a bounded acceleration, so motion is smooth and within envelope. The
easing reuses [`Ramp`] as its slew-rate limiter, with the step set to `accel * dt` each update.

**Examples**

```
use pamoja_kit::{Limits, Twist};

// Up to 1 m/s and 2 rad/s, easing on at 0.5 m/s^2 and 4 rad/s^2.
let mut limits = Limits::new(1.0, 2.0, 0.5, 4.0);

// Asking for full speed from rest: the first 0.1 s step is acceleration-limited.
let cmd = limits.apply(Twist::planar(1.0, 0.0), 0.1);
assert!((cmd.vx - 0.05).abs() < 1e-6); // 0.5 m/s^2 * 0.1 s
```

### `Limits::new`

Creates limits from the speed and acceleration ceilings.

**Arguments**

* `max_linear` - the largest planar speed; its magnitude is used.
* `max_angular` - the largest yaw rate; its magnitude is used.
* `max_linear_accel` - the largest change in linear speed per second; its magnitude is used.
* `max_angular_accel` - the largest change in yaw rate per second; its magnitude is used.

**Returns**

Limits starting from rest.

```rust
fn new(max_linear: f32, max_angular: f32, max_linear_accel: f32, max_angular_accel: f32,) -> Self
```

### `Limits::reset`

Resets the remembered motion to rest, so the next command eases up from zero.

```rust
fn reset(&mut self)
```

### `Limits::apply`

Bounds a desired command in speed and acceleration and returns the safe command.

**Arguments**

* `desired` - the requested body motion.
* `dt` - the time since the previous call, setting the acceleration step.

**Returns**

The command after capping speed and easing toward it within the acceleration limit.

```rust
fn apply(&mut self, desired: Twist, dt: f32) -> Twist
```

## struct `SafetyGate`

The single gate every motion command passes through, composing e-stop, watchdog, and limits.

This is the one call a control loop makes to drive safely: feed it the desired motion and the
time step, and it returns what is actually safe to command. It stops hard (commands zero and
forgets its motion history, so resuming eases from rest) whenever the e-stop is engaged or the
watchdog has expired; otherwise it returns the desired motion bounded by the [`Limits`]. Call
[`feed`](SafetyGate::feed) whenever a fresh command arrives to keep the watchdog satisfied.

**Examples**

```
use pamoja_kit::{Limits, SafetyGate, Twist};

let limits = Limits::new(1.0, 2.0, 0.5, 4.0);
let mut gate = SafetyGate::new(limits, 0.2); // stop if unfed for 0.2 s

gate.feed();
let cmd = gate.command(Twist::planar(1.0, 0.0), 0.1);
assert!((cmd.vx - 0.05).abs() < 1e-6); // eased on, acceleration-limited

gate.engage_estop();
assert_eq!(gate.command(Twist::planar(1.0, 0.0), 0.1), Twist::zero());
```

### `SafetyGate::new`

Creates a gate from motion limits and a watchdog timeout.

**Arguments**

* `limits` - the speed and acceleration bounds for normal motion.
* `watchdog_timeout` - the allowed silence before the gate stops the robot.

**Returns**

The gate, cleared and freshly fed.

```rust
fn new(limits: Limits, watchdog_timeout: f32) -> Self
```

### `SafetyGate::feed`

Feeds the watchdog; call this whenever a fresh command arrives.

```rust
fn feed(&mut self)
```

### `SafetyGate::engage_estop`

Engages the latching emergency stop.

```rust
fn engage_estop(&mut self)
```

### `SafetyGate::reset_estop`

Clears the emergency stop.

```rust
fn reset_estop(&mut self)
```

### `SafetyGate::is_stopped`

Returns whether the gate is currently forcing a stop.

**Returns**

`true` if the e-stop is engaged or the watchdog has expired.

```rust
fn is_stopped(&self) -> bool
```

### `SafetyGate::command`

Returns the safe command for a desired motion over a time step.

**Arguments**

* `desired` - the requested body motion.
* `dt` - the time since the previous call.

**Returns**

[`Twist::zero`] when stopped, otherwise the desired motion bounded by the limits.

```rust
fn command(&mut self, desired: Twist, dt: f32) -> Twist
```

