# pamoja-kit::thermostat

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

Keeping a reading near a setpoint with on/off control.

## struct `Thermostat`

A hysteresis (bang-bang) controller for a single on/off actuator.

This is the controller behind "keep a temperature". It switches a cooler or
heater on and off to hold a reading near a setpoint. A deadband around the
setpoint - the hysteresis - stops the output chattering when the reading hovers
at the threshold, which protects relays and compressors that have a limited
number of switching cycles in them.

**Examples**

```
use pamoja_kit::Thermostat;

let mut fridge = Thermostat::cooling(4.0, 0.5);
assert!(fridge.update(5.0)); // above the deadband: the cooler runs
assert!(fridge.update(4.2)); // inside the deadband: it holds its state
assert!(!fridge.update(3.4)); // below the deadband: the cooler stops
```

### `Thermostat::cooling`

Creates a thermostat that drives a cooler, such as a fridge.

The output turns on when the reading rises above the deadband and off when
it falls below it.

**Arguments**

* `setpoint` - the target reading.
* `hysteresis` - half the deadband width; its magnitude is used.

**Returns**

A thermostat whose output starts off.

```rust
fn cooling(setpoint: f32, hysteresis: f32) -> Self
```

### `Thermostat::heating`

Creates a thermostat that drives a heater.

The output turns on when the reading falls below the deadband and off when
it rises above it.

**Arguments**

* `setpoint` - the target reading.
* `hysteresis` - half the deadband width; its magnitude is used.

**Returns**

A thermostat whose output starts off.

```rust
fn heating(setpoint: f32, hysteresis: f32) -> Self
```

### `Thermostat::update`

Updates the controller with a reading and returns whether the output is on.

**Arguments**

* `reading` - the latest measured value.

**Returns**

`true` if the cooler or heater should be running.

```rust
fn update(&mut self, reading: f32) -> bool
```

### `Thermostat::is_on`

Returns whether the output is currently on.

**Returns**

`true` if the most recent [`update`](Self::update) left the output running.

```rust
fn is_on(&self) -> bool
```

