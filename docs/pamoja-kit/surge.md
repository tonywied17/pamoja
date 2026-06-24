# pamoja-kit::surge

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

Catching a value that moves dangerously fast.

## struct `Surge`

Warns when a reading changes faster than a safe rate.

This is the primitive behind "warn me before it is too late": a river level
rising fast enough to mean a flash flood, a gas reading spiking toward an
explosive level, or a tank pressure collapsing. Feed it successive readings and
it reports the rate whenever the change since the previous sample, in the
direction being watched, exceeds a limit. The technique one layer down is a
first difference between consecutive samples, so a noisy signal pairs well with a
[`Smoother`](crate::Smoother) on the input to avoid false alarms.

**Examples**

```
use pamoja_kit::Surge;

// A river gauge in metres, sampled each minute: alarm if it rises faster than
// 0.5 m per sample.
let mut flood = Surge::rising(0.5);
assert_eq!(flood.update(1.0), None); // first reading: no rate yet
assert_eq!(flood.update(1.25), None); // a gentle rise is fine
assert_eq!(flood.update(2.0), Some(0.75)); // a 0.75 m jump: a flash flood
```

### `Surge::rising`

Creates an alarm for a value rising too fast.

**Arguments**

* `limit` - the largest safe increase per sample; its magnitude is used.

**Returns**

An alarm awaiting its first reading.

```rust
fn rising(limit: f32) -> Self
```

### `Surge::falling`

Creates an alarm for a value falling too fast.

**Arguments**

* `limit` - the largest safe decrease per sample; its magnitude is used.

**Returns**

An alarm awaiting its first reading.

```rust
fn falling(limit: f32) -> Self
```

### `Surge::update`

Records a reading and reports the rate if it changed too fast.

**Arguments**

* `value` - the latest reading.

**Returns**

`Some(rate)` for the change since the previous sample when it exceeds the limit
in the watched direction, where `rate` is that change as a positive number;
`None` if the change is within the limit, is in the other direction, or this is
the first reading.

```rust
fn update(&mut self, value: f32) -> Option <f32>
```

