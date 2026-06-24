# pamoja-kit::navigate

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

Steering a robot toward a waypoint, and stopping before an obstacle.

## struct `Guidance`

The steering command toward a waypoint, with the geometry behind it.

Fields:

- `twist: Twist` - The body twist to drive: a forward speed and a yaw rate toward the target.
- `distance_m: f64` - The remaining distance to the target, in metres.
- `heading_error_deg: f32` - The heading error to the target, in degrees, in `(-180, 180]`.
- `arrived: bool` - Whether the target is within the arrival radius.

## struct `WaypointFollower`

Guides a robot from waypoint to waypoint by GPS-style coordinates (carrot following).

This is the "go to that point" primitive behind a patrol route, a return-to-base, or a field
pass: given where the robot is, which way it faces, and the next waypoint, it produces the
twist to get there. It turns toward the target in proportion to the heading error and slows
the forward speed as that error grows (by the cosine of the error), so the robot pivots toward
a target behind it before driving off, rather than swinging wide. The caller holds the list of
waypoints and advances to the next once [`Guidance::arrived`] is set, which keeps this
allocation-free.

**Examples**

```
use pamoja_kit::{Coordinate, WaypointFollower};

// Cruise 1.5 m/s, arrive within 3 m, turn at 1.5 rad per rad of error, cap 1 rad/s.
let follower = WaypointFollower::new(1.5, 3.0, 1.5, 1.0);

// At the equator/prime meridian facing east (90 deg), with the target due east.
let here = Coordinate::new(0.0, 0.0);
let target = Coordinate::new(0.0, 0.01);
let g = follower.guide(here, 90.0, target);
assert!(g.heading_error_deg.abs() < 1e-3); // already pointed at it
assert!((g.twist.vx - 1.5).abs() < 1e-3); // so drive at cruise
assert!(!g.arrived);
```

### `WaypointFollower::new`

Creates a follower with the given speeds and tolerances.

**Arguments**

* `cruise` - the forward speed when pointed at the target; its magnitude is used.
* `arrival_m` - how close, in metres, counts as arrived; its magnitude is used.
* `heading_gain` - yaw rate commanded per radian of heading error; its magnitude is used.
* `max_angular` - the largest yaw rate to command; its magnitude is used.

**Returns**

The follower.

```rust
fn new(cruise: f32, arrival_m: f64, heading_gain: f32, max_angular: f32) -> Self
```

### `WaypointFollower::guide`

Produces the steering command from the robot's position and heading to a target.

**Arguments**

* `here` - the robot's current coordinate.
* `heading_deg` - the robot's heading in degrees clockwise from north (a compass course).
* `target` - the waypoint to head toward.

**Returns**

The [`Guidance`]; once within the arrival radius the twist is zero and `arrived` is set.

```rust
fn guide(&self, here: Coordinate, heading_deg: f32, target: Coordinate) -> Guidance
```

## fn `obstacle_stop`

Stops forward motion when an obstacle is within the stopping distance, leaving turning free.

This is the simplest reliable safety reflex for a robot with a forward range sensor: hold the
requested rotation so the robot can still turn away, but cut `vx` and `vy` to zero once the
nearest reading falls inside the stop distance, so it does not drive into what it sees.

**Arguments**

* `twist` - the requested body motion.
* `range_m` - the nearest measured range ahead, in metres.
* `stop_distance_m` - the range at or below which forward motion is cut; its magnitude is used.

**Returns**

The original twist when the way is clear, or one with no translation (rotation preserved) when
an obstacle is within range.

**Examples**

```
use pamoja_kit::{obstacle_stop, Twist};

let driving = Twist::new(1.0, 0.0, 0.5);
// Clear ahead: unchanged.
assert_eq!(obstacle_stop(driving, 2.0, 0.5), driving);
// Obstacle at 0.3 m: forward cut, turn kept so it can escape.
assert_eq!(obstacle_stop(driving, 0.3, 0.5), Twist::new(0.0, 0.0, 0.5));
```

```rust
fn obstacle_stop(twist: Twist, range_m: f32, stop_distance_m: f32) -> Twist
```

