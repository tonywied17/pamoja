# pamoja-kit::imu

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

Tilt from a three-axis accelerometer.

## struct `Tilt`

Roll and pitch angles, in degrees.

Fields:

- `roll: f64` - Rotation about the forward (x) axis, in degrees, in `[-180.0, 180.0]`.
- `pitch: f64` - Rotation about the right (y) axis, in degrees, in `[-90.0, 90.0]`.

## fn `tilt_from_accel`

Computes roll and pitch from a three-axis accelerometer reading.

At rest, gravity tells a three-axis accelerometer which way is down, which fixes the
board's tilt. This uses the standard formula (Freescale AN3461): roll is `atan2(ay, az)`
and pitch is `atan2(-ax, sqrt(ay^2 + az^2))`, with `atan2` placing each angle in the
correct quadrant. The reading's units do not matter (raw counts or g), because only the
ratios between axes set the angle and any common scale cancels. It holds while the board
is still or moving gently, since it assumes the only acceleration is gravity. Yaw
(heading) cannot be found from an accelerometer alone; that needs a magnetometer.

**Arguments**

* `ax` - acceleration along the x (forward) axis.
* `ay` - acceleration along the y (right) axis.
* `az` - acceleration along the z (up) axis.

**Returns**

The [`Tilt`] in degrees.

**Examples**

```
use pamoja_kit::imu::tilt_from_accel;

// Board level, 1 g straight down on z: no tilt.
let level = tilt_from_accel(0.0, 0.0, 1.0);
assert!(level.roll.abs() < 1e-6 && level.pitch.abs() < 1e-6);

// Tipped fully onto its y axis: 90 degrees of roll.
let rolled = tilt_from_accel(0.0, 1.0, 0.0);
assert!((rolled.roll - 90.0).abs() < 1e-6);
```

```rust
fn tilt_from_accel(ax: f64, ay: f64, az: f64) -> Tilt
```

