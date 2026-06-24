# pamoja-kit::arm

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

Serial-arm (manipulator) kinematics: where the hand is, and how to place it.

A robot arm is a chain of joints, and two questions recur: given the joint angles, where is the
tool (forward kinematics), and given a target, what joint angles put the tool there (inverse
kinematics). [`forward_kinematics`] answers the first for any serial arm described in the
standard Denavit-Hartenberg convention, in full 3D. The second has no closed form for a general
arm, so this provides the classic solvable case, the planar [`TwoLinkArm`], with both its
elbow-up and elbow-down solutions; numeric inverse kinematics for longer chains can build on the
same forward model later.

## struct `Transform`

A 4x4 homogeneous transform: a rotation and a translation in one matrix.

Stored row-major, this is the building block of forward kinematics: each joint contributes one
transform, and chaining them places the tool relative to the base.

Fields:

- `m: [f32 ; 16]` - The sixteen elements in row-major order (row 0 first).

### `Transform::identity`

Returns the identity transform: no rotation, no translation.

**Returns**

The identity.

```rust
fn identity() -> Self
```

### `Transform::multiply`

Returns the product `self * other`, the transform that applies `other` then `self`.

**Arguments**

* `other` - the transform applied first (the one further down the chain).

**Returns**

The composed transform.

```rust
fn multiply(&self, other: &Transform) -> Transform
```

### `Transform::position`

Returns the translation part: the position this transform places the origin at.

**Returns**

`(x, y, z)`, the last column of the matrix.

```rust
fn position(&self) ->(f32, f32, f32)
```

## struct `DhParameters`

The four Denavit-Hartenberg parameters describing one joint-to-joint step of a serial arm.

The DH convention pins each link with four numbers, so an arm is just a list of these. For a
revolute joint the joint variable is `theta`; for a prismatic joint it is `d`.

Fields:

- `a: f32` - Link length: distance along the common normal, in metres.
- `alpha: f32` - Link twist: angle about the common normal, in radians.
- `d: f32` - Link offset: distance along the previous z axis, in metres.
- `theta: f32` - Joint angle: rotation about the previous z axis, in radians.

### `DhParameters::transform`

Returns the homogeneous [`Transform`] for this DH step.

**Returns**

The standard DH transform built from `(a, alpha, d, theta)`.

```rust
fn transform(&self) -> Transform
```

## fn `forward_kinematics`

Returns the transform from the base to the tool for a serial arm of DH joints.

**Arguments**

* `joints` - the arm's joints, base first, each as [`DhParameters`].

**Returns**

The composed base-to-tool [`Transform`]; the identity for an empty arm. Take
[`Transform::position`] for the tool point.

**Examples**

```
use pamoja_kit::{forward_kinematics, DhParameters};

// A two-link planar arm written in DH form: links of 1.0, both joints at 0, flat along x.
let arm = [
    DhParameters { a: 1.0, alpha: 0.0, d: 0.0, theta: 0.0 },
    DhParameters { a: 1.0, alpha: 0.0, d: 0.0, theta: 0.0 },
];
let (x, y, _z) = forward_kinematics(&arm).position();
assert!((x - 2.0).abs() < 1e-5 && y.abs() < 1e-5); // reaches straight out to x = 2
```

```rust
fn forward_kinematics(joints: &[DhParameters]) -> Transform
```

## enum `Elbow`

Which way a two-link arm's elbow bends; both reach the same point.

- `Up` - The elbow bends so the second joint angle is positive (counter-clockwise).
- `Down` - The elbow bends so the second joint angle is negative (clockwise).

## struct `TwoLinkArm`

A planar two-link arm: the textbook arm with a closed-form inverse.

Two links of fixed length in a plane, with a shoulder and an elbow joint. [`tip`](TwoLinkArm::tip)
is forward kinematics; [`joints_for`](TwoLinkArm::joints_for) is the analytic inverse, returning
the shoulder and elbow angles that place the hand at a target, for the chosen [`Elbow`] branch.

**Examples**

```
use pamoja_kit::{Elbow, TwoLinkArm};

let arm = TwoLinkArm::new(1.0, 1.0);
// Place the hand, then recover the joint angles for it.
let (x, y) = arm.tip(0.5, 0.7);
let (q1, q2) = arm.joints_for(x, y, Elbow::Up).unwrap();
assert!((q1 - 0.5).abs() < 1e-4 && (q2 - 0.7).abs() < 1e-4);

// A target beyond the arm's reach has no solution.
assert!(arm.joints_for(5.0, 0.0, Elbow::Up).is_none());
```

### `TwoLinkArm::new`

Creates an arm from its two link lengths.

**Arguments**

* `l1` - the first (shoulder) link length; its magnitude is used.
* `l2` - the second (elbow) link length; its magnitude is used.

**Returns**

The arm.

```rust
fn new(l1: f32, l2: f32) -> Self
```

### `TwoLinkArm::reach`

Returns the closest and farthest distances the hand can reach from the shoulder.

**Returns**

`(min, max)`, where `min` is `|l1 - l2|` and `max` is `l1 + l2`.

```rust
fn reach(&self) ->(f32, f32)
```

### `TwoLinkArm::tip`

Returns the hand position for given joint angles (forward kinematics).

**Arguments**

* `shoulder` - the first joint angle, in radians from the x axis.
* `elbow` - the second joint angle, in radians relative to the first link.

**Returns**

The hand `(x, y)`.

```rust
fn tip(&self, shoulder: f32, elbow: f32) ->(f32, f32)
```

### `TwoLinkArm::joints_for`

Returns the joint angles that place the hand at a target (inverse kinematics).

**Arguments**

* `x` - the target x coordinate.
* `y` - the target y coordinate.
* `elbow` - which [`Elbow`] branch to solve for.

**Returns**

`Some((shoulder, elbow))` for a reachable target, or `None` if the target lies outside the
arm's reach.

```rust
fn joints_for(&self, x: f32, y: f32, elbow: Elbow) -> Option <(f32, f32)>
```

