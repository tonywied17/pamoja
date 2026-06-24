# pamoja-kit::drive

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

Differential-drive wheel kinematics.

## struct `DiffDrive`

Converts between a robot's motion and its two wheel speeds (differential drive).

A differential-drive robot steers by spinning its left and right wheels at different
speeds. This converts both ways: [`wheel_speeds`](DiffDrive::wheel_speeds) turns a desired
forward speed and turn rate into the wheel speeds to command (inverse kinematics), and
[`body_motion`](DiffDrive::body_motion) turns measured wheel speeds back into the robot's
forward speed and turn rate (forward kinematics). The one parameter is the track: the
distance between the wheels.

**Examples**

```
use pamoja_kit::DiffDrive;

let drive = DiffDrive::new(0.5); // wheels 0.5 apart
// Drive straight: both wheels turn at the forward speed.
assert_eq!(drive.wheel_speeds(1.0, 0.0), (1.0, 1.0));
// Spin in place: the wheels turn opposite, each at turn rate times half the track.
assert_eq!(drive.wheel_speeds(0.0, 2.0), (-0.5, 0.5));
```

### `DiffDrive::new`

Creates a model for wheels `track` apart.

**Arguments**

* `track` - the distance between the left and right wheels; its magnitude is used.

**Returns**

The kinematics model.

```rust
fn new(track: f32) -> Self
```

### `DiffDrive::wheel_speeds`

Returns the `(left, right)` wheel speeds for a desired body motion.

**Arguments**

* `linear` - the forward speed.
* `angular` - the turn rate, positive turning toward the left (counter-clockwise).

**Returns**

`(left, right)`, where `left = linear - angular * track / 2` and
`right = linear + angular * track / 2`.

```rust
fn wheel_speeds(&self, linear: f32, angular: f32) ->(f32, f32)
```

### `DiffDrive::body_motion`

Returns the body `(linear, angular)` motion for measured wheel speeds.

**Arguments**

* `left` - the left wheel speed.
* `right` - the right wheel speed.

**Returns**

`(linear, angular)`, where `linear = (right + left) / 2` and
`angular = (right - left) / track`. Angular is zero when the track is zero.

```rust
fn body_motion(&self, left: f32, right: f32) ->(f32, f32)
```

