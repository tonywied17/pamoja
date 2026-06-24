# pamoja-kit::chassis

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

Wheel kinematics for the common mobile-robot chassis layouts.

[`DiffDrive`](crate::DiffDrive) covers the two-wheel differential robot. This module adds
the other layouts a builder is likely to meet: a car-like [`Ackermann`] steer, a tracked or
four-wheel [`SkidSteer`], and a sideways-capable [`Mecanum`] base. Each converts both ways,
turning a desired body motion into wheel commands (inverse kinematics) and measured wheel
motion back into the body's velocity (forward kinematics), so the same model drives the
robot and reads its odometry.

## struct `Ackermann`

Car-like (Ackermann) steering: one steered axle and a driven axle a wheelbase apart.

A car cannot turn in place; it follows an arc whose radius is set by the steering angle. The
kinematic bicycle model collapses each axle to a single central wheel, so the steering angle
`delta`, the wheelbase `L`, the forward speed `v`, and the yaw rate `omega` are related by
`tan(delta) = L * omega / v`, equivalently a turn radius `R = L / tan(delta)`. This is the
standard model behind a rover, a tractor, or any rack-and-pinion vehicle.

**Examples**

```
use pamoja_kit::Ackermann;

// A rover with a 2.5 m wheelbase, steering 30 degrees (about 0.5236 rad).
let car = Ackermann::new(2.5);
let radius = car.turn_radius(0.5236);
assert!((radius - 4.33).abs() < 0.01); // R = 2.5 / tan(30 deg)

// At 5 m/s that steering yields a yaw rate of about 1.155 rad/s.
let omega = car.yaw_rate(5.0, 0.5236);
assert!((omega - 1.1547).abs() < 1e-3);
// And asking for that yaw rate at that speed recovers the steering angle.
assert!((car.steering_angle(5.0, omega) - 0.5236).abs() < 1e-3);
```

### `Ackermann::new`

Creates a model for a vehicle whose axles are `wheelbase` apart.

**Arguments**

* `wheelbase` - the distance from the steered axle to the driven axle; its magnitude
  is used.

**Returns**

The kinematics model.

```rust
fn new(wheelbase: f32) -> Self
```

### `Ackermann::steering_angle`

Returns the steering angle for a desired forward speed and yaw rate.

**Arguments**

* `linear` - the forward speed.
* `angular` - the desired yaw rate, positive turning left.

**Returns**

The steering angle in radians, `atan(wheelbase * angular / linear)`. Returns zero when
the vehicle is stopped (`linear` is zero), since a stationary car cannot yaw by steering.

```rust
fn steering_angle(&self, linear: f32, angular: f32) -> f32
```

### `Ackermann::yaw_rate`

Returns the yaw rate produced by a forward speed and steering angle.

**Arguments**

* `linear` - the forward speed.
* `steering` - the steering angle in radians.

**Returns**

The yaw rate `linear * tan(steering) / wheelbase`, zero when the wheelbase is zero.

```rust
fn yaw_rate(&self, linear: f32, steering: f32) -> f32
```

### `Ackermann::turn_radius`

Returns the turn radius for a steering angle.

**Arguments**

* `steering` - the steering angle in radians.

**Returns**

The radius `wheelbase / tan(steering)` in metres, or [`f32::INFINITY`] when the wheels
point straight ahead (`steering` is zero), since the path is then a straight line.

```rust
fn turn_radius(&self, steering: f32) -> f32
```

### `Ackermann::curvature`

Returns the path curvature for a steering angle.

**Arguments**

* `steering` - the steering angle in radians.

**Returns**

The curvature `tan(steering) / wheelbase`, the reciprocal of the turn radius, zero when
the wheelbase is zero.

```rust
fn curvature(&self, steering: f32) -> f32
```

## struct `SkidSteer`

Skid-steer (tracked or four-wheel) drive: turning by spinning each side at a different speed.

A skid-steer robot steers like a differential one, but its wheels or tracks must slip
sideways to turn, so the geometric track under-predicts the turn. The standard correction is
an effective track wider than the real one by a `slip` factor (at least one), found by
calibration: commanding a yaw rate needs a larger left-right speed difference than the bare
geometry suggests. With `slip` of one this reduces to plain differential drive.

**Examples**

```
use pamoja_kit::SkidSteer;

// Wheels 0.5 m apart that slip enough to need a 1.2x wider effective track.
let drive = SkidSteer::new(0.5, 1.2);
// Spin in place at 2 rad/s: each side runs at omega * effective_track / 2.
let (left, right) = drive.wheel_speeds(0.0, 2.0);
assert!((left + 0.6).abs() < 1e-6 && (right - 0.6).abs() < 1e-6);
// Reading the wheels back recovers the body motion.
let (linear, angular) = drive.body_motion(left, right);
assert!(linear.abs() < 1e-6 && (angular - 2.0).abs() < 1e-6);
```

### `SkidSteer::new`

Creates a model for wheels `track` apart with a given `slip` factor.

**Arguments**

* `track` - the distance between the left and right wheels or tracks; its magnitude is
  used.
* `slip` - how much wider the effective track is than the geometric one; its magnitude
  is used, and a value of zero is treated as one (no slip).

**Returns**

The kinematics model.

```rust
fn new(track: f32, slip: f32) -> Self
```

### `SkidSteer::wheel_speeds`

Returns the `(left, right)` wheel speeds for a desired body motion.

**Arguments**

* `linear` - the forward speed.
* `angular` - the yaw rate, positive turning left.

**Returns**

`(left, right)`, where the split uses the effective (slip-corrected) track.

```rust
fn wheel_speeds(&self, linear: f32, angular: f32) ->(f32, f32)
```

### `SkidSteer::body_motion`

Returns the body `(linear, angular)` motion for measured wheel speeds.

**Arguments**

* `left` - the left wheel speed.
* `right` - the right wheel speed.

**Returns**

`(linear, angular)`, where `angular` divides by the effective track and is zero when
that track is zero.

```rust
fn body_motion(&self, left: f32, right: f32) ->(f32, f32)
```

## struct `WheelSpeeds`

The four wheel speeds of a mecanum or omni base, front and rear, left and right.

Each value is the speed of that wheel's contact point in the same units as the body
velocity, positive when the wheel drives the robot forward.

Fields:

- `front_left: f32` - Front-left wheel speed.
- `front_right: f32` - Front-right wheel speed.
- `rear_left: f32` - Rear-left wheel speed.
- `rear_right: f32` - Rear-right wheel speed.

## struct `Mecanum`

Mecanum (four-wheel omnidirectional) drive: forward, sideways, and turning at once.

A mecanum base carries four wheels whose angled rollers let it strafe sideways as well as
drive and turn, so it tracks a full planar [`Twist`] (`vx`, `vy`, `omega`). This uses the
standard kinematics for the common "O" roller layout, with `k = (half-wheelbase +
half-track)`:

```text
front_left  = vx - vy - k*omega      rear_left  = vx + vy - k*omega
front_right = vx + vy + k*omega      rear_right = vx - vy + k*omega
```

The forward kinematics invert these by averaging the wheels. Speeds are wheel contact
speeds in the body's units; convert to motor rates by dividing by the wheel radius.

**Examples**

```
use pamoja_kit::{Mecanum, Twist, WheelSpeeds};

// 0.4 m wheelbase, 0.3 m track.
let base = Mecanum::new(0.4, 0.3);

// Pure left strafe: the O-layout spins the diagonals against each other.
let w = base.wheel_speeds(Twist::new(0.0, 1.0, 0.0));
assert_eq!(w, WheelSpeeds { front_left: -1.0, front_right: 1.0, rear_left: 1.0, rear_right: -1.0 });

// Reading the wheels back recovers the body twist.
let t = base.body_motion(w);
assert!(t.vx.abs() < 1e-6 && (t.vy - 1.0).abs() < 1e-6 && t.omega.abs() < 1e-6);
```

### `Mecanum::new`

Creates a model from the wheelbase and track.

**Arguments**

* `wheelbase` - the front-to-rear distance between axles; its magnitude is used.
* `track` - the left-to-right distance between wheels; its magnitude is used.

**Returns**

The kinematics model.

```rust
fn new(wheelbase: f32, track: f32) -> Self
```

### `Mecanum::wheel_speeds`

Returns the four wheel speeds for a desired body twist.

**Arguments**

* `twist` - the desired body velocity, using all of `vx`, `vy`, and `omega`.

**Returns**

The [`WheelSpeeds`] that produce that twist.

```rust
fn wheel_speeds(&self, twist: Twist) -> WheelSpeeds
```

### `Mecanum::body_motion`

Returns the body twist for measured wheel speeds.

**Arguments**

* `wheels` - the four measured wheel speeds.

**Returns**

The body [`Twist`]; `omega` is zero when the base has no size (lever arm zero).

```rust
fn body_motion(&self, wheels: WheelSpeeds) -> Twist
```

