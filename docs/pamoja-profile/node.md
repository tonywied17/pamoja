# pamoja-profile::node

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

The ready-to-run node a profile assembles around real components.

## struct `NoActuator`

An actuator that accepts and ignores commands.

Profiles that only observe - such as a well-level monitor - have no output to
drive. [`Node::monitor`] wires this in their place so a node has one uniform
shape whether or not it switches an actuator.

## struct `Node`

A profile assembled around the components that make it run.

The node is the thin shell that ties a [`Profile`]'s decision logic to real I/O.
Each [`tick`](Node::tick) reads the sensor, runs the profile's
[`Controller`](crate::Controller), drives the actuator when the controller calls
for it, and publishes the reading over the transport with the supplied codec. The
control math lives in `pamoja-kit`, the power schedule in `pamoja-power`, and the
wire format in the codec, so the node adds composition, not behavior.

Readings are real-world `f32` units (degrees, percent, litres), the form the
`pamoja-kit` controllers expect; a driver is responsible for calibrating raw
counts into those units before the node sees them.

**Examples**

```
use pamoja_codec::CborCodec;
use pamoja_core::{Actuator, Result, Sensor, Transport};
use pamoja_loopback::{LoopbackBroker, LoopbackTransport};
use pamoja_profile::{Node, Profile};

struct Probe(f32);
impl Sensor for Probe {
    type Reading = f32;
    async fn read(&mut self) -> Result<f32> {
        Ok(self.0)
    }
}

struct Cooler;
impl Actuator for Cooler {
    type Command = bool;
    async fn apply(&mut self, _on: bool) -> Result<()> {
        Ok(())
    }
}

let broker = LoopbackBroker::new();
let mut link = LoopbackTransport::new(broker);
link.connect().await?;

// A warm fridge, assembled straight from its profile.
let mut node = Node::new(Profile::vaccine_fridge_monitor(), Probe(9.0), Cooler, link, CborCodec);
let reaction = node.tick().await?;
assert_eq!(reaction.actuator, Some(true)); // the cooler runs
assert!(reaction.alert.is_some()); // and 9 C is a spoilage excursion
```

### `Node <S, A, T, C>::new`

Assembles a node from a profile and the components that drive it.

**Arguments**

* `profile` - the profile to assemble; its policy becomes the node's controller.
* `sensor` - the source of readings.
* `actuator` - the output the controller switches.
* `transport` - the link readings are published over; expected to be connected.
* `codec` - the wire format readings are encoded with.

**Returns**

A node ready to [`tick`](Node::tick).

```rust
fn new(profile: Profile, sensor: S, actuator: A, transport: T, codec: C) -> Self
```

### `Node <S, A, T, C>::profile`

Returns the profile this node was assembled from.

**Returns**

A reference to the node's [`Profile`].

```rust
fn profile(&self) -> &Profile
```

### `Node <S, A, T, C>::schedule`

Returns the power mode and wait interval for the next cycle.

This assembles the profile's [`PowerSchedule`](crate::PowerSchedule) into a
`pamoja-power` governor: as the battery drains the interval stretches, and a
charging panel eases the node back toward its active cadence. The node never
sleeps; the caller waits the
returned [`Duration`] before the next [`tick`](Node::tick), so timing stays
outside the node and the decision logic remains synchronous and testable.

**Arguments**

* `soc` - the battery state of charge in `[0.0, 1.0]`.
* `charging` - whether the panel is currently delivering charge.

**Returns**

The [`PowerMode`] to run in and how long to wait before the next cycle.

```rust
fn schedule(&self, soc: f32, charging: bool) ->(PowerMode, Duration)
```

### `Node <S, NoActuator, T, C>::monitor`

Assembles a node for a profile that observes without driving an output.

Wires a [`NoActuator`] in place of a real output, so a monitoring profile such
as [`well_level`](Profile::well_level) reads and publishes with the same shape
as a controlling one.

**Arguments**

* `profile` - the profile to assemble.
* `sensor` - the source of readings.
* `transport` - the link readings are published over; expected to be connected.
* `codec` - the wire format readings are encoded with.

**Returns**

A node ready to [`tick`](Node::tick), with no output to switch.

```rust
fn monitor(profile: Profile, sensor: S, transport: T, codec: C) -> Self
```

### `Node <S, A, T, C>::tick`

Runs one read-decide-act-publish cycle.

Reads the sensor, evaluates the profile's controller, applies the resulting
command to the actuator when the controller calls for one, and publishes the
reading to the profile's topic.

**Returns**

The [`Reaction`] the controller produced: the actuator setting that was
applied (if any) and any alert the reading raised.

**Errors**

Returns [`Error::Io`](pamoja_core::Error::Io) or
[`Error::Closed`](pamoja_core::Error::Closed) if the sensor read or the
actuator command fails, [`Error::Codec`](pamoja_core::Error::Codec) if the
reading cannot be encoded, and [`Error::Transport`](pamoja_core::Error::Transport)
or [`Error::Closed`](pamoja_core::Error::Closed) if the publish fails.

```rust
async fn tick(&mut self) -> Result <Reaction>
```

