# pamoja-kit::motion

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

Shared planar motion types: a body twist and a world pose.

## struct `Twist`

A planar body velocity: the command a wheeled robot is driven with.

The frame follows the robotics convention (ROS REP-103): x points forward, y points to
the robot's left, and a positive `omega` turns counter-clockwise (toward the left).
Nonholonomic drives (differential, Ackermann, skid-steer) cannot move sideways and ignore
`vy`; holonomic drives (mecanum, omni) use it.

**Examples**

```
use pamoja_kit::Twist;

let forward = Twist::planar(1.0, 0.0); // 1 m/s ahead, no turn
assert_eq!(forward.vy, 0.0);
assert_eq!(Twist::zero(), Twist::new(0.0, 0.0, 0.0));
```

Fields:

- `vx: f32` - Forward speed along the x axis.
- `vy: f32` - Leftward speed along the y axis; zero for drives that cannot strafe.
- `omega: f32` - Yaw rate about the z axis, positive counter-clockwise.

### `Twist::new`

Creates a twist from its three components.

**Arguments**

* `vx` - forward speed.
* `vy` - leftward speed.
* `omega` - yaw rate, positive counter-clockwise.

**Returns**

The twist.

```rust
fn new(vx: f32, vy: f32, omega: f32) -> Self
```

### `Twist::planar`

Creates a planar twist with no sideways motion (`vy = 0`).

**Arguments**

* `vx` - forward speed.
* `omega` - yaw rate, positive counter-clockwise.

**Returns**

The twist with `vy` zero.

```rust
fn planar(vx: f32, omega: f32) -> Self
```

### `Twist::zero`

Returns the zero twist: stopped.

**Returns**

A twist whose every component is zero.

```rust
fn zero() -> Self
```

## struct `Pose`

A planar pose in the world frame: position and heading.

**Examples**

```
use pamoja_kit::Pose;

let start = Pose::origin();
assert_eq!((start.x, start.y, start.theta), (0.0, 0.0, 0.0));
```

Fields:

- `x: f32` - Position along the world x axis, in metres.
- `y: f32` - Position along the world y axis, in metres.
- `theta: f32` - Heading from the world x axis, in radians, in `(-pi, pi]`, positive counter-clockwise.

### `Pose::new`

Creates a pose; the heading is wrapped into `(-pi, pi]`.

**Arguments**

* `x` - position along the world x axis.
* `y` - position along the world y axis.
* `theta` - heading in radians, wrapped to `(-pi, pi]`.

**Returns**

The pose.

```rust
fn new(x: f32, y: f32, theta: f32) -> Self
```

### `Pose::origin`

Returns the origin pose: at `(0, 0)` facing along the x axis.

**Returns**

The origin pose.

```rust
fn origin() -> Self
```

