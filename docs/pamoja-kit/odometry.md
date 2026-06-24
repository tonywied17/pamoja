# pamoja-kit::odometry

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

Dead-reckoning a robot's pose from its motion.

## struct `Odometry`

Tracks a robot's [`Pose`] by accumulating its motion over time (odometry).

With no GPS indoors, a robot estimates where it is by adding up where it has been: each small
move is integrated onto the running pose. This uses the exact arc model rather than a straight-
line step, so a robot that drives and turns at once follows the curve it actually traces
instead of cutting the corner; over many steps that is markedly more accurate. Feed it either a
body motion (forward speed and yaw rate over a time step) or wheel-distance deltas through a
[`DiffDrive`] model. Dead reckoning drifts, so correct the heading from an absolute source with
[`fuse_heading`](Odometry::fuse_heading) when one is available.

**Examples**

```
use pamoja_kit::{Odometry, Pose};

// Drive a quarter circle of radius 1 m: forward 1 m/s, turning left at 1 rad/s, for pi/2 s.
let mut odom = Odometry::at_origin();
let pose = odom.integrate(1.0, 1.0, core::f32::consts::FRAC_PI_2);

// It ends about (1, 1) facing 90 degrees, the far corner of the arc.
assert!((pose.x - 1.0).abs() < 1e-5);
assert!((pose.y - 1.0).abs() < 1e-5);
assert!((pose.theta - core::f32::consts::FRAC_PI_2).abs() < 1e-5);
```

### `Odometry::new`

Creates an estimator starting from a known pose.

**Arguments**

* `start` - the initial pose.

**Returns**

The estimator.

```rust
fn new(start: Pose) -> Self
```

### `Odometry::at_origin`

Creates an estimator starting at the origin facing along the x axis.

**Returns**

The estimator.

```rust
fn at_origin() -> Self
```

### `Odometry::pose`

Returns the current pose estimate.

**Returns**

The pose accumulated so far.

```rust
fn pose(&self) -> Pose
```

### `Odometry::reset`

Resets the estimate to a known pose.

**Arguments**

* `pose` - the pose to set.

```rust
fn reset(&mut self, pose: Pose)
```

### `Odometry::integrate`

Integrates a body motion over a time step and returns the new pose.

**Arguments**

* `linear` - the forward speed.
* `angular` - the yaw rate, positive turning left.
* `dt` - the length of the time step.

**Returns**

The updated pose.

```rust
fn integrate(&mut self, linear: f32, angular: f32, dt: f32) -> Pose
```

### `Odometry::integrate_wheels`

Integrates wheel-distance deltas through a differential-drive model.

**Arguments**

* `left` - the distance the left wheel rolled since the last update.
* `right` - the distance the right wheel rolled since the last update.
* `drive` - the [`DiffDrive`] model giving the track between the wheels.

**Returns**

The updated pose. The wheel deltas are turned into a forward distance and a heading
change by [`DiffDrive::body_motion`], then integrated as one arc.

```rust
fn integrate_wheels(&mut self, left: f32, right: f32, drive: &DiffDrive) -> Pose
```

### `Odometry::fuse_heading`

Corrects the heading toward an absolute measurement, the way a compass tames gyro drift.

This is the angular cousin of [`Complementary`](crate::Complementary): it nudges the
estimated heading along the shortest arc toward an absolute reading (an IMU yaw, a
magnetometer, a GPS course) by a blend weight, leaving the position untouched.

**Arguments**

* `measured` - the absolute heading in radians.
* `weight` - how strongly to trust the measurement, clamped to `[0, 1]`; zero keeps the
  dead-reckoned heading, one snaps to `measured`.

```rust
fn fuse_heading(&mut self, measured: f32, weight: f32)
```

