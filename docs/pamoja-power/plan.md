# pamoja-power::plan

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

An energy-aware governor that adapts the work cadence to the battery.

## enum `PowerMode`

How hard a node should work, chosen from its battery state of charge.

- `Active` - Healthy charge: run at the normal cadence.
- `Saver` - Low charge: stretch the cadence to conserve.
- `Critical` - Critically low charge: do the bare minimum to survive.

## struct `PowerPlan`

Maps a battery state of charge onto a [`PowerMode`] and a work interval.

As the battery drains, a node should do less: sample and transmit less often so
it survives the night or a cloudy week. A [`PowerPlan`] encodes that policy as
three intervals and two thresholds. Feed it a state of charge in `[0.0, 1.0]`
and it returns the mode to run in and how long to wait before the next cycle.
When the panel is charging it eases off by one mode, since incoming energy buys
back some headroom.

**Examples**

```
use core::time::Duration;
use pamoja_power::{PowerMode, PowerPlan};

let plan = PowerPlan::new(
    Duration::from_secs(60),
    Duration::from_secs(600),
    Duration::from_secs(3600),
);

// Low battery means the saver cadence...
assert_eq!(plan.mode(0.3), PowerMode::Saver);
// ...unless the panel is charging, which buys back the active cadence.
assert_eq!(plan.mode_while_charging(0.3, true), PowerMode::Active);
```

### `PowerPlan::new`

Creates a plan from its three work intervals, with default thresholds.

The defaults enter [`PowerMode::Saver`] below 50% charge and
[`PowerMode::Critical`] below 20%.

**Arguments**

* `active` - the interval at a healthy charge.
* `saver` - the longer interval used to conserve, normally larger than
  `active`.
* `critical` - the longest interval, used when charge is critically low.

**Returns**

The power plan.

```rust
fn new(active: Duration, saver: Duration, critical: Duration) -> Self
```

### `PowerPlan::thresholds`

Sets the state-of-charge thresholds for entering each lower mode.

**Arguments**

* `saver_below` - enter [`PowerMode::Saver`] when charge is below this.
* `critical_below` - enter [`PowerMode::Critical`] when charge is below this,
  normally lower than `saver_below`.

**Returns**

The updated plan, for chaining.

```rust
fn thresholds(mut self, saver_below: f32, critical_below: f32) -> Self
```

### `PowerPlan::mode`

Returns the mode for the given state of charge.

**Arguments**

* `soc` - the battery state of charge in `[0.0, 1.0]`.

**Returns**

The [`PowerMode`] the node should run in.

```rust
fn mode(&self, soc: f32) -> PowerMode
```

### `PowerPlan::mode_while_charging`

Returns the mode for the given charge, easing off by one step when charging.

**Arguments**

* `soc` - the battery state of charge in `[0.0, 1.0]`.
* `charging` - whether the panel is currently delivering charge.

**Returns**

The [`PowerMode`], promoted one step toward [`PowerMode::Active`] while
`charging` is `true`.

```rust
fn mode_while_charging(&self, soc: f32, charging: bool) -> PowerMode
```

### `PowerPlan::interval_for`

Returns the work interval for a given mode.

**Arguments**

* `mode` - the mode to look up.

**Returns**

The interval to wait before the next work cycle in that mode.

```rust
fn interval_for(&self, mode: PowerMode) -> Duration
```

### `PowerPlan::interval`

Returns the work interval for the given state of charge.

**Arguments**

* `soc` - the battery state of charge in `[0.0, 1.0]`.

**Returns**

The interval to wait before the next work cycle.

```rust
fn interval(&self, soc: f32) -> Duration
```

