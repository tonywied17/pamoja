# pamoja-kit::drivers

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

The small, exact conversions between robot actuators and sensors and real units.

Driving a robot means turning intent into the pulse widths a servo or motor controller expects,
and turning encoder edges back into how far a wheel has rolled. Each conversion is pure
arithmetic with a classic off-by-one or sign trap, so it lives here as checked logic rather
than scattered inline math: a [`ServoMap`] and [`Esc`] for hobby PWM outputs, and a
[`Quadrature`] decoder with a [`QuadratureScale`] for incremental encoders. Clocking the pulses
and reading the pins arrives with the hardware-I/O layer; this is the math ahead of it.

## struct `ServoMap`

Maps a servo angle to its RC pulse width in microseconds, and back.

A hobby servo is positioned by the width of a pulse repeated about every 20 ms: the standard
range is 1000 to 2000 microseconds spanning the full travel, with 1500 at centre.
[`ServoMap::standard`] uses those defaults over 180 degrees; [`ServoMap::new`] covers servos
with a different range or travel.

**Examples**

```
use pamoja_kit::ServoMap;

let servo = ServoMap::standard();
assert_eq!(servo.pulse(0.0), 1000);
assert_eq!(servo.pulse(90.0), 1500); // centre
assert_eq!(servo.pulse(180.0), 2000);
assert!((servo.angle(1500) - 90.0).abs() < 1e-3);
```

### `ServoMap::standard`

Returns the standard hobby-servo map: 1000 to 2000 microseconds over 180 degrees.

**Returns**

The standard map.

```rust
fn standard() -> Self
```

### `ServoMap::new`

Creates a map from explicit pulse and travel limits.

**Arguments**

* `min_us` - the pulse width at zero degrees.
* `max_us` - the pulse width at full travel.
* `range_deg` - the full travel in degrees; its magnitude is used.

**Returns**

The map.

```rust
fn new(min_us: u16, max_us: u16, range_deg: f32) -> Self
```

### `ServoMap::pulse`

Returns the pulse width for an angle.

**Arguments**

* `angle_deg` - the desired angle in degrees, clamped to `[0, range]`.

**Returns**

The pulse width in microseconds.

```rust
fn pulse(&self, angle_deg: f32) -> u16
```

### `ServoMap::angle`

Returns the angle for a pulse width.

**Arguments**

* `pulse_us` - the pulse width in microseconds, clamped to the configured range.

**Returns**

The angle in degrees, zero when the pulse range is empty.

```rust
fn angle(&self, pulse_us: u16) -> f32
```

## struct `Esc`

Maps a normalized throttle to an electronic speed controller's RC pulse width.

An ESC reads the same RC pulse a servo does, with the pulse width setting motor output.
[`Esc::bidirectional`] uses the common reversible scheme: 1000 microseconds full reverse, 1500
neutral, 2000 full forward, so a throttle in `[-1, 1]` maps linearly across it.

**Examples**

```
use pamoja_kit::Esc;

let esc = Esc::bidirectional();
assert_eq!(esc.pulse(0.0), 1500); // neutral
assert_eq!(esc.pulse(1.0), 2000); // full forward
assert_eq!(esc.pulse(-1.0), 1000); // full reverse
assert_eq!(esc.pulse(0.5), 1750);
```

### `Esc::bidirectional`

Returns the standard reversible ESC map: 1000 / 1500 / 2000 microseconds.

**Returns**

The bidirectional map.

```rust
fn bidirectional() -> Self
```

### `Esc::new`

Creates a map from explicit reverse, neutral, and forward pulse widths.

**Arguments**

* `min_us` - the pulse width at full reverse.
* `neutral_us` - the pulse width at rest.
* `max_us` - the pulse width at full forward.

**Returns**

The map.

```rust
fn new(min_us: u16, neutral_us: u16, max_us: u16) -> Self
```

### `Esc::pulse`

Returns the pulse width for a throttle.

**Arguments**

* `throttle` - the demand in `[-1, 1]`, clamped; negative reverses, positive drives forward.

**Returns**

The pulse width in microseconds.

```rust
fn pulse(&self, throttle: f32) -> u16
```

## struct `Quadrature`

Decodes a quadrature (A/B) encoder into a running tick count.

An incremental encoder reports motion as two square waves a quarter-cycle apart; their order of
change tells direction. Feeding successive A/B readings to [`update`](Quadrature::update) returns
the per-step direction and accumulates a signed count, the foundation for wheel odometry. Pair
it with a [`QuadratureScale`] to turn that count into metres.

**Examples**

```
use pamoja_kit::Quadrature;

let mut enc = Quadrature::new();
// One full cycle forward: 00 -> 01 -> 11 -> 10 -> 00, one tick each.
for &(a, b) in &[(false, true), (true, true), (true, false), (false, false)] {
    assert_eq!(enc.update(a, b), 1);
}
assert_eq!(enc.count(), 4);
```

### `Quadrature::new`

Creates a decoder assuming both channels start low.

**Returns**

A decoder with a zero count.

```rust
fn new() -> Self
```

### `Quadrature::starting`

Creates a decoder seeded with the encoder's current channel levels.

Seeding the initial state avoids a spurious first tick when the encoder does not happen to
rest with both channels low.

**Arguments**

* `a` - the current A channel level.
* `b` - the current B channel level.

**Returns**

A decoder with a zero count and the given starting state.

```rust
fn starting(a: bool, b: bool) -> Self
```

### `Quadrature::update`

Feeds the latest channel levels and returns the tick delta.

**Arguments**

* `a` - the latest A channel level.
* `b` - the latest B channel level.

**Returns**

`+1` or `-1` for a step in either direction, or `0` for no change or an illegal jump.

```rust
fn update(&mut self, a: bool, b: bool) -> i8
```

### `Quadrature::count`

Returns the accumulated signed tick count.

**Returns**

The running count.

```rust
fn count(&self) -> i64
```

### `Quadrature::reset`

Resets the count to zero, keeping the current channel state.

```rust
fn reset(&mut self)
```

## struct `QuadratureScale`

Converts encoder ticks into the distance and speed a wheel has travelled.

**Examples**

```
use pamoja_kit::QuadratureScale;

// 360 ticks per revolution on a wheel of 0.05 m radius.
let scale = QuadratureScale::new(360.0, 0.05);
// One full revolution rolls out one circumference.
assert!((scale.distance(360) - (2.0 * core::f32::consts::PI * 0.05)).abs() < 1e-6);
```

### `QuadratureScale::new`

Creates a scale from the encoder resolution and wheel size.

**Arguments**

* `counts_per_rev` - ticks per wheel revolution; its magnitude is used.
* `wheel_radius` - the wheel radius in metres; its magnitude is used.

**Returns**

The scale.

```rust
fn new(counts_per_rev: f32, wheel_radius: f32) -> Self
```

### `QuadratureScale::distance`

Returns the distance rolled for a tick count.

**Arguments**

* `count` - the signed tick count.

**Returns**

The distance in metres, zero when the resolution is zero.

```rust
fn distance(&self, count: i64) -> f32
```

### `QuadratureScale::velocity`

Returns the speed for a tick count accumulated over a time step.

**Arguments**

* `delta_count` - the ticks counted during the step.
* `dt` - the length of the step.

**Returns**

The speed in metres per second, zero when `dt` is zero.

```rust
fn velocity(&self, delta_count: i64, dt: f32) -> f32
```

