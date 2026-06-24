# pamoja-dashboard::mock

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

A scenario-driven mock fleet, so the whole dashboard runs with no hardware.

The mock is the heart of the hardware-free development workflow. It implements
[`StateSource`] and serves a believable, deterministic multi-organization fleet, so
every state the UI must handle can be reproduced on demand instead of waited for in
the field. Readings drift on slow sine waves (real sensors wander gently rather than
jumping), and a [`Scenario`] injects a condition - an alarm, a sensor fault, a flat
battery, a dropped link, a cold start - into the otherwise healthy fleet so each
state is one click away.

## enum `Scenario`

A reproducible condition injected into the fleet for the dashboard to render.

Selectable from the command line and switchable live with `?scenario=`, so one
running dev server covers every case.

- `Normal` - The whole fleet is healthy.
- `Alarm` - A cold-chain fridge has drifted out of its safe band.
- `SensorFault` - A silo probe has failed and reads an impossible value.
- `LowBattery` - A solar microgrid's batteries are nearly flat.
- `LinkLost` - A river-watch group has lost its uplink.
- `ColdStart` - The fleet has just booted, with little history yet.

### `Scenario::key`

Returns the stable query-parameter key for this scenario.

**Returns**

The lowercase identifier used in `?scenario=` and on the command line.

```rust
fn key(self) -> &'static str
```

### `Scenario::from_key`

Parses a scenario from its [`key`](Scenario::key).

**Arguments**

* `key` - the scenario identifier, as used in `?scenario=`.

**Returns**

The matching scenario, or `None` if `key` names none.

```rust
fn from_key(key: &str) -> Option <Scenario>
```

## struct `Mock`

A deterministic, hardware-free fleet that serves a [`Scenario`].

Create one with [`Mock::new`], poll it through [`StateSource::snapshot`], and flip
scenarios live with [`Mock::set_scenario`]. Readings drift smoothly and repeatably,
because they are sine waves of the tick rather than real randomness.

**Examples**

```
use pamoja_dashboard::{Mock, Scenario, StateSource, Status};

let mut fleet = Mock::new(Scenario::Alarm);
let state = fleet.snapshot();
assert_eq!(state.status, Status::Alarm);
assert!(!state.orgs.is_empty());
```

### `Mock::new`

Creates a mock running `scenario` at tick zero.

**Arguments**

* `scenario` - the condition to inject into the fleet.

**Returns**

A mock with a deterministic drift sequence.

```rust
fn new(scenario: Scenario) -> Self
```

### `Mock::set_scenario`

Switches the scenario served from the next snapshot on.

**Arguments**

* `scenario` - the new condition to inject.

```rust
fn set_scenario(&mut self, scenario: Scenario)
```

### `Mock::scenario`

Returns the scenario currently being served.

**Returns**

The active scenario.

```rust
fn scenario(&self) -> Scenario
```

