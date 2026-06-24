# pamoja-profile::control

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

The decision logic a profile assembles: turning a reading into a reaction.

## enum `Alert`

An alert raised when a reading crosses a profile's safety threshold.

- `OutOfRange` - A controlled reading drifted outside its safe band.  For a cold-chain fridge this is a spoilage excursion: the cooler may be running, but the contents are no longer within the safe temperature range.
- `RunningOut` - A falling level will reach its empty mark within this many more samples.
- `ChangingFast` - A reading is changing faster than its safe rate.  For a river gauge this is a flash-flood warning: the level jumped further in one sample than the profile allows.

## struct `Reaction`

The outcome of evaluating one reading against a profile's control policy.

Fields:

- `actuator: Option <bool>` - The actuator setting this reading calls for, if the profile drives one.  `Some(true)` switches the output on, `Some(false)` switches it off, and `None` means the profile observes without driving an output.
- `alert: Option <Alert>` - An alert, if the reading crossed a profile threshold; `None` otherwise.

## struct `Controller`

The assembled, stateful decision logic of a profile.

A controller is what a [`Profile`](crate::Profile) turns its
[`ControlSpec`](crate::ControlSpec) into: the live loop that maps each reading to
a [`Reaction`]. It composes the `pamoja-kit` helpers - a
[`Thermostat`](pamoja_kit::Thermostat) for on/off control, a
[`Depletion`](pamoja_kit::Depletion) predictor for level alerts, and a
[`Surge`](pamoja_kit::Surge) alarm for rapid change - so the same field-tested
math drives every profile. The logic is synchronous and
hardware-free, so a profile's whole control policy is unit-testable with no
devices and no network.

**Examples**

```
use pamoja_profile::{Alert, Controller};

// Hold a fridge near 5 C, alerting if it strays more than 3 C from target.
let mut control = Controller::setpoint(5.0, 0.5, true, 3.0);

let reaction = control.evaluate(9.0); // warm and out of the safe band
assert_eq!(reaction.actuator, Some(true));
assert!(matches!(reaction.alert, Some(Alert::OutOfRange { .. })));
```

### `Controller::setpoint`

Builds a controller that holds a reading near a setpoint.

This is the policy behind "keep a temperature" and "keep the soil watered":
it switches an output on and off around the setpoint and raises an
[`Alert::OutOfRange`] when the reading strays beyond `safe_band`.

**Arguments**

* `setpoint` - the target reading.
* `hysteresis` - half the deadband width around the setpoint, which stops the
  output chattering at the threshold.
* `cooling` - `true` for an output that switches on above the band (a cooler),
  `false` for one that switches on below it (a heater or an irrigation valve).
* `safe_band` - how far the reading may stray from the setpoint before an
  alert fires.

**Returns**

A controller whose output starts off.

```rust
fn setpoint(setpoint: f32, hysteresis: f32, cooling: bool, safe_band: f32) -> Self
```

### `Controller::level`

Builds a controller that warns before a falling level runs out.

This is the policy behind "warn before a tank runs dry": it watches a level
fall and raises an [`Alert::RunningOut`] once it is estimated to reach `empty`
within `warn_within` more samples.

**Arguments**

* `empty` - the level treated as empty, such as a dry tank.
* `warn_within` - warn once empty is this many samples away or nearer.

**Returns**

A controller awaiting its first two readings.

```rust
fn level(empty: f32, warn_within: u32) -> Self
```

### `Controller::surge`

Builds a controller that warns when a reading changes too fast.

This is the policy behind "warn me before it is too late": it watches the
change between samples and raises an [`Alert::ChangingFast`] when a reading
moves more than `limit` per sample in the watched direction, such as a river
level rising into a flash flood.

**Arguments**

* `rising` - watch a rapid rise (`true`) or a rapid fall (`false`).
* `limit` - the largest safe change per sample.

**Returns**

A controller awaiting its first reading.

```rust
fn surge(rising: bool, limit: f32) -> Self
```

### `Controller::monitor`

Builds a controller that reports readings without driving an output.

**Returns**

A controller that never commands an actuator and never alerts.

```rust
fn monitor() -> Self
```

### `Controller::evaluate`

Evaluates one reading and returns the action and any alert it calls for.

**Arguments**

* `reading` - the latest measured value, in real-world units.

**Returns**

The [`Reaction`] for this reading: the actuator setting (if the profile drives
one) and any alert the reading raised.

```rust
fn evaluate(&mut self, reading: f32) -> Reaction
```

