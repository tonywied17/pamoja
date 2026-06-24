# pamoja-sim::robot

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

A hardware-free mobile robot you can drive and watch move.

## struct `SimRobot`

A simulated differential-drive robot: drive it with a [`Twist`], read back its [`Pose`].

This stands in for a real rover in a hardware-free test or demo. It is both an
[`Actuator`] whose command is a body twist and a [`Sensor`] whose reading is the pose:
each command advances the robot one time step at the commanded velocity, integrating the
motion with the same exact-arc odometry a real robot would use, and a read returns where it
has reached. A control loop can therefore be developed and tested end to end with no robot.

**Examples**

```
use pamoja_core::{Actuator, Sensor};
use pamoja_kit::Twist;
use pamoja_sim::SimRobot;

let mut robot = SimRobot::new(0.1); // 0.1 s per command
// Drive straight at 1 m/s for ten steps: about one metre forward.
for _ in 0..10 {
    robot.apply(Twist::planar(1.0, 0.0)).await?;
}
let pose = robot.read().await?;
assert!((pose.x - 1.0).abs() < 1e-5 && pose.y.abs() < 1e-5);
```

### `SimRobot::new`

Creates a robot at the origin that advances `dt` seconds per command.

**Arguments**

* `dt` - the time each [`apply`](SimRobot::apply) advances the robot; its magnitude is used.

**Returns**

The simulated robot.

```rust
fn new(dt: f32) -> Self
```

### `SimRobot::starting_at`

Creates a robot starting from a known pose.

**Arguments**

* `pose` - the starting pose.
* `dt` - the time each command advances the robot; its magnitude is used.

**Returns**

The simulated robot.

```rust
fn starting_at(pose: Pose, dt: f32) -> Self
```

### `SimRobot::pose`

Returns the robot's current pose.

**Returns**

The pose reached so far.

```rust
fn pose(&self) -> Pose
```

