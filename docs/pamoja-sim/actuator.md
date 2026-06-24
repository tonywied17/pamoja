# pamoja-sim::actuator

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

A fake actuator that records the commands it is given.

## struct `RecordingActuator`

An actuator that records every command instead of driving hardware.

This stands in for a relay, a valve, or a motor in a hardware-free test: it
implements the core [`Actuator`] trait and keeps an ordered log of every command
applied to it, so a test can assert what a control loop decided to do. Take a
[`log`](RecordingActuator::log) handle before moving the actuator into a `Node`,
then read the commands back through it afterwards.

**Examples**

```
use pamoja_core::Actuator;
use pamoja_sim::RecordingActuator;

let mut relay = RecordingActuator::new();
let log = relay.log();

relay.apply(true).await?;
relay.apply(false).await?;

assert_eq!(log.commands(), vec![true, false]);
```

### `RecordingActuator <C>::new`

Creates an actuator with an empty command log.

**Returns**

A recording actuator.

```rust
fn new() -> Self
```

### `RecordingActuator <C>::log`

Returns a handle that reads this actuator's command log.

The handle shares the same underlying log, so commands applied after it is
taken are still visible through it.

**Returns**

An [`ActuatorLog`] over the same recorded commands.

```rust
fn log(&self) -> ActuatorLog <C>
```

## struct `ActuatorLog`

A read handle over a [`RecordingActuator`]'s command log.

### `ActuatorLog <C>::commands`

Returns a snapshot of every command applied so far, in order.

**Returns**

A copy of the recorded commands.

```rust
fn commands(&self) -> Vec <C>
```

### `ActuatorLog <C>::len`

Returns how many commands have been applied.

**Returns**

The number of recorded commands.

```rust
fn len(&self) -> usize
```

### `ActuatorLog <C>::is_empty`

Returns whether no command has been applied yet.

**Returns**

`true` if the command log is empty.

```rust
fn is_empty(&self) -> bool
```

