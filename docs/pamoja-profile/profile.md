# pamoja-profile::profile

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

The profile manifest and its named, ready-to-run presets.

A profile is data: a [`Profile`] serializes to and from a manifest a community
can write by hand, store in a file, and share. The presets here are convenience
constructors for the same data, not a closed set - any manifest that names a
[`ControlSpec`] and a [`PowerSchedule`] is a valid profile.

## enum `ControlSpec`

How a profile turns each reading into control output and alerts.

This is the policy half of a profile's manifest: the tunable rule a community can
publish and share, with no code to write. [`Profile::controller`] assembles it
into a live [`Controller`]. In a manifest it is tagged by `kind`:

```json
{ "kind": "setpoint", "setpoint": 5.0, "hysteresis": 0.5, "cooling": true, "safe_band": 3.0 }
```

- `Setpoint` - Hold a reading near `setpoint` by switching an output on and off.
- `Level` - Watch a falling level and warn before it reaches `empty`.
- `Surge` - Warn when a reading changes faster than `limit` per sample.
- `Monitor` - Report readings only, with no control output and no alerts.

## struct `PowerSchedule`

How often a node samples as its battery drains, in plain seconds.

This is the serializable form of a [`PowerPlan`](pamoja_power::PowerPlan): a
manifest carries the three work intervals as whole seconds and the two
state-of-charge thresholds, and [`plan`](PowerSchedule::plan) assembles the
`pamoja-power` governor from them. The thresholds may be omitted from a manifest,
in which case they default to entering the saver cadence below 50% charge and the
critical cadence below 20%.

Fields:

- `active_secs: u64` - Seconds between samples at a healthy charge.
- `saver_secs: u64` - Seconds between samples while conserving.
- `critical_secs: u64` - Seconds between samples when critically low.
- `saver_below: f32` - Enter the saver cadence below this state of charge.
- `critical_below: f32` - Enter the critical cadence below this state of charge.

### `PowerSchedule::new`

Creates a schedule from its three work intervals, with default thresholds.

**Arguments**

* `active_secs` - seconds between samples at a healthy charge.
* `saver_secs` - seconds between samples while conserving.
* `critical_secs` - seconds between samples when critically low.

**Returns**

A schedule that enters the saver cadence below 50% charge and the critical
cadence below 20%.

```rust
fn new(active_secs: u64, saver_secs: u64, critical_secs: u64) -> Self
```

### `PowerSchedule::with_thresholds`

Sets the state-of-charge thresholds for entering each lower cadence.

**Arguments**

* `saver_below` - enter the saver cadence when charge is below this.
* `critical_below` - enter the critical cadence when charge is below this,
  normally lower than `saver_below`.

**Returns**

The updated schedule, for chaining.

```rust
fn with_thresholds(mut self, saver_below: f32, critical_below: f32) -> Self
```

### `PowerSchedule::plan`

Assembles the `pamoja-power` governor this schedule describes.

**Returns**

A [`PowerPlan`](pamoja_power::PowerPlan) with this schedule's intervals and
thresholds.

```rust
fn plan(&self) -> PowerPlan
```

## struct `Profile`

A named, pre-wired bundle of control policy, publish topic, and power schedule.

A profile is the unit a builder instantiates instead of wiring pins and tuning
constants, and it is plain data: it serializes to and from a manifest a community
can write, store in a file, and share. Pick a preset such as
[`vaccine_fridge_monitor`](Profile::vaccine_fridge_monitor) or load one with
[`from_json`](Profile::from_json), hand it a sensor, an actuator, a transport, and
a codec, and the resulting [`Node`](crate::Node) reads, decides, drives the
output, and publishes on its own. Every field is public, so a deployment can
adjust the policy, topic, or power schedule in place.

**Examples**

```
use pamoja_profile::{ControlSpec, Profile};

let profile = Profile::vaccine_fridge_monitor();
assert_eq!(profile.name, "vaccine-fridge-monitor");
assert!(matches!(profile.control, ControlSpec::Setpoint { .. }));
```

Fields:

- `name: String` - A stable, human-readable name, such as `"vaccine-fridge-monitor"`.
- `topic: String` - The topic each reading is published to.
- `control: ControlSpec` - The control policy applied to each reading.
- `power: PowerSchedule` - The power schedule that sets how often the node samples as the battery drains.

### `Profile::vaccine_fridge_monitor`

A cold-chain fridge monitor: hold 5 C and alert on a spoilage excursion.

Switches a cooler to hold the contents near 5 C and raises an
[`Alert::OutOfRange`](crate::Alert::OutOfRange) the moment the temperature
leaves the 2-8 C safe range. Data integrity outweighs power here, so it keeps
sampling often even as the battery drains.

**Returns**

The cold-chain monitoring profile.

```rust
fn vaccine_fridge_monitor() -> Self
```

### `Profile::irrigation_node`

An irrigation node: hold soil moisture near a target by opening a valve.

Treats the valve as a "heater" for soil moisture, opening it when the soil
dries below the band and closing it once it is wet enough, and alerts if the
soil falls critically dry. Samples less often than the fridge, since soil
changes slowly and battery life matters more.

**Returns**

The irrigation profile.

```rust
fn irrigation_node() -> Self
```

### `Profile::well_level`

A well-level monitor: report depth and warn before the well runs dry.

Observes the water level without driving an output and raises an
[`Alert::RunningOut`](crate::Alert::RunningOut) once the level is on course to
reach the dry mark within a few more samples.

**Returns**

The well-level monitoring profile.

```rust
fn well_level() -> Self
```

### `Profile::flood_sensor`

A flash-flood sensor: warn when a river level rises dangerously fast.

Watches a river or stream gauge and raises an
[`Alert::ChangingFast`](crate::Alert::ChangingFast) when the level rises more
than 0.3 m in a single sample, the signature of a flash flood. It samples
often, because a flood gives little warning.

**Returns**

The flash-flood monitoring profile.

**Examples**

```
use pamoja_profile::{Alert, Profile};

let mut control = Profile::flood_sensor().controller();
control.evaluate(1.0); // first fix establishes the level
let reaction = control.evaluate(1.5); // the river jumped 0.5 m
assert!(matches!(reaction.alert, Some(Alert::ChangingFast { .. })));
```

```rust
fn flood_sensor() -> Self
```

### `Profile::controller`

Assembles this profile's [`ControlSpec`] into a live [`Controller`].

**Returns**

A fresh controller implementing the profile's policy, with its control state
reset.

```rust
fn controller(&self) -> Controller
```

### `Profile::from_json`

Loads a profile from a JSON manifest.

This is how a shared profile reaches a device: a community publishes a manifest
file, and the runtime loads it into a profile to assemble a node from.

**Arguments**

* `manifest` - the JSON text of the profile.

**Returns**

The profile described by `manifest`.

**Errors**

Returns [`Error::Codec`](pamoja_core::Error::Codec) if `manifest` is not valid
JSON or does not describe a profile.

**Examples**

```
use pamoja_profile::Profile;

// A well-level monitor, shared as a manifest. The power thresholds are
// optional and default when omitted.
let manifest = r#"{
    "name": "tank-level",
    "topic": "water/tank/level",
    "control": { "kind": "level", "empty": 0.0, "warn_within": 5 },
    "power": { "active_secs": 600, "saver_secs": 1800, "critical_secs": 3600 }
}"#;

let profile = Profile::from_json(manifest).expect("valid manifest");
assert_eq!(profile.name, "tank-level");

let mut control = profile.controller();
control.evaluate(10.0); // first reading establishes a level
assert!(control.evaluate(2.0).alert.is_some()); // falling fast toward empty
```

```rust
fn from_json(manifest: &str) -> pamoja_core::Result <Self>
```

### `Profile::to_json`

Serializes this profile to a JSON manifest a community can share.

**Returns**

The pretty-printed JSON text of the profile.

**Errors**

Returns [`Error::Codec`](pamoja_core::Error::Codec) if the profile cannot be
serialized.

```rust
fn to_json(&self) -> pamoja_core::Result <String>
```

