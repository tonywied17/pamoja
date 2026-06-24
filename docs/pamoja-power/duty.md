# pamoja-power::duty

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

Duty cycling: trading wakefulness for battery life.

## struct `DutyCycle`

A repeating wake/sleep schedule.

Duty cycling is the simplest way to make a battery or solar node last: stay
awake just long enough to do the work, then sleep through the rest of the
period. The duty fraction - the share of each period spent awake - is a good
first proxy for average power draw, so roughly halving it halves the energy the
cycle costs.

**Examples**

```
use core::time::Duration;
use pamoja_power::DutyCycle;

// Wake for one second every minute.
let cycle = DutyCycle::new(Duration::from_secs(1), Duration::from_secs(59));
assert_eq!(cycle.period(), Duration::from_secs(60));
assert!((cycle.fraction() - 1.0 / 60.0).abs() < 1e-6);
```

### `DutyCycle::new`

Creates a duty cycle from its awake and asleep durations.

**Arguments**

* `active` - how long to stay awake each period.
* `sleep` - how long to sleep each period.

**Returns**

The duty cycle.

```rust
fn new(active: Duration, sleep: Duration) -> Self
```

### `DutyCycle::from_fraction`

Creates a duty cycle from a period and the fraction of it to stay awake.

**Arguments**

* `period` - the full wake-plus-sleep period.
* `fraction` - the share of the period to stay awake, clamped to
  `[0.0, 1.0]`.

**Returns**

A duty cycle whose awake time is `fraction` of `period`.

```rust
fn from_fraction(period: Duration, fraction: f32) -> Self
```

### `DutyCycle::active`

Returns the awake portion of each period.

```rust
fn active(&self) -> Duration
```

### `DutyCycle::sleep`

Returns the asleep portion of each period.

```rust
fn sleep(&self) -> Duration
```

### `DutyCycle::period`

Returns the full period, awake plus asleep.

```rust
fn period(&self) -> Duration
```

### `DutyCycle::fraction`

Returns the share of each period spent awake, in `[0.0, 1.0]`.

**Returns**

The duty fraction, or `0.0` for a zero-length period.

```rust
fn fraction(&self) -> f32
```

