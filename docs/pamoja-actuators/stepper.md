# pamoja-actuators::stepper

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

Coil sequencing for four-wire stepper motors.

A stepper turns by energising its coils in a repeating pattern; stepping through
the pattern in one direction or the other advances or reverses the shaft. This
module holds the three standard drive patterns and a sequencer that walks them, so
a caller toggles the four coil lines (directly, or through a darlington array like
the ULN2003) without hand-maintaining the sequence. It also models the
step-and-direction interface of driver chips such as the A4988 and DRV8825, which
take a step pulse and a direction level and track position as a signed count.

Coil patterns are four bits, the most significant being the first coil (IN1 on a
ULN2003 board) down to the least significant (IN4).

## enum `Direction`

Which way to step.

- `Forward` - Advance the sequence, turning the shaft one way.
- `Backward` - Reverse the sequence, turning the shaft the other way.

## enum `Drive`

A stepper drive pattern.

The three patterns trade torque, smoothness, and resolution. Wave drive energises
one coil at a time (least torque, least power); full-step energises two at a time
(most torque); half-step alternates between them to double the resolution at the
cost of uneven torque.

- `Wave` - One coil energised at a time: four steps.
- `FullStep` - Two adjacent coils energised at a time: four steps, more torque.
- `HalfStep` - Alternating one and two coils: eight steps, double resolution.

### `Drive::pattern`

Returns the coil patterns for this drive, in forward order.

**Returns**

A slice of four-bit coil patterns: four for wave and full-step, eight for
half-step.

```rust
fn pattern(self) -> &'static [u8]
```

### `Drive::step_count`

Returns how many steps make up one full electrical cycle of this drive.

**Returns**

`4` for wave and full-step, `8` for half-step.

```rust
fn step_count(self) -> usize
```

## struct `Sequencer`

A position in a drive sequence, walked one step at a time.

Holding the index into the pattern, a sequencer turns each [`step`](Sequencer::step)
into the next coil pattern to apply, wrapping around the cycle so it can run
indefinitely in either direction.

**Examples**

```
use pamoja_actuators::stepper::{Direction, Drive, Sequencer};

let mut sequencer = Sequencer::new(Drive::HalfStep);
assert_eq!(sequencer.coils(), 0b1000);
assert_eq!(sequencer.step(Direction::Forward), 0b1100);

// One full electrical cycle returns to the start.
for _ in 1..Drive::HalfStep.step_count() {
    sequencer.step(Direction::Forward);
}
assert_eq!(sequencer.coils(), 0b1000);
```

### `Sequencer::new`

Creates a sequencer at the start of a drive pattern.

**Arguments**

* `drive` - the drive pattern to walk.

**Returns**

A sequencer whose current pattern is the first in the drive.

```rust
fn new(drive: Drive) -> Sequencer
```

### `Sequencer::coils`

Returns the coil pattern at the current position.

**Returns**

The four-bit coil pattern to apply now.

```rust
fn coils(&self) -> u8
```

### `Sequencer::step`

Advances one step in `direction` and returns the new coil pattern.

The index wraps around the cycle, so stepping forever in either direction is
well defined.

**Arguments**

* `direction` - which way to step.

**Returns**

The coil pattern to apply after the step.

```rust
fn step(&mut self, direction: Direction) -> u8
```

### `Sequencer::drive`

Returns the drive pattern this sequencer walks.

```rust
fn drive(&self) -> Drive
```

## struct `Position`

A signed step counter for step-and-direction driver chips.

Driver chips like the A4988 and DRV8825 move one (micro)step per pulse in the
direction set on a level pin, so the host just counts. This tracks that count, so
position is known without reading the hardware.

### `Position::new`

Creates a position at zero.

**Returns**

A position whose step count is zero.

```rust
fn new() -> Position
```

### `Position::step`

Records one step in `direction` and returns the new step count.

**Arguments**

* `direction` - which way the pulse moved the motor.

**Returns**

The updated step count, increasing for [`Direction::Forward`].

```rust
fn step(&mut self, direction: Direction) -> i32
```

### `Position::steps`

Returns the current step count.

```rust
fn steps(self) -> i32
```

## fn `steps_for_degrees`

Converts an angle to a whole number of steps for a given motor.

**Arguments**

* `degrees` - the angle to turn; negative turns the other way.
* `steps_per_revolution` - the motor's steps per full turn, for example 200 for a
  1.8-degree motor.

**Returns**

The nearest whole number of steps to that angle.

```rust
fn steps_for_degrees(degrees: f32, steps_per_revolution: u32) -> i32
```

