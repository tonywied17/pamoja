# pamoja-dashboard::fleet

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

A real fleet source: a project pushes readings in, the dashboard reads them out, and
control commands queue back for the project to apply.

The dashboard renders whatever implements [`StateSource`]. [`Mock`](crate::Mock) is the
hardware-free demo; `Fleet` is the real one. Sensing in this SDK is async and the server
is synchronous, so the project owns its own sampling loop - ticking its profiles and
nodes on their power schedule - and pushes each result into a `Fleet` with the report
methods. The dashboard reads the latest with [`snapshot`](StateSource::snapshot), and
authenticated control [`Command`]s queue for the project to drain with
[`take_commands`](Fleet::take_commands) and apply to real hardware, then reflect the
result back through the report methods. The device stays authoritative; provisioning and
actuation are also applied optimistically so the UI updates at once.

A `Fleet` is cheap to [`Clone`] (it shares one inner state), so one handle drives the
[`Server`](crate::Server) while another stays with the sampling loop.

**Examples**

```
use pamoja_dashboard::{Fleet, LinkKind, Reading, Sensor, StateSource, Status};

let fleet = Fleet::builder()
    .org("clinic", "Kano clinic")
    .group("clinic", "fridges", "Cold chain", LinkKind::Cellular)
    .sensor(
        "fridges",
        Sensor::new("fridge-1", Reading::new("fridge_temp", 4.5, "celsius").with_band(2.0, 8.0)),
    )
    .build();

// The sampling loop pushes a fresh reading; the dashboard sees it.
fleet.report_reading(
    "fridges",
    "fridge-1",
    Reading::new("fridge_temp", 9.2, "celsius").with_band(2.0, 8.0).with_status(Status::Alarm),
);
let mut handle = fleet.clone();
assert_eq!(handle.snapshot().status, Status::Alarm);
```

## struct `Fleet`

A real fleet a project fills and the dashboard renders. Clone to share one between the
serving layer and the sampling loop.

### `Fleet::builder`

Starts building a fleet's initial structure.

**Returns**

An empty [`FleetBuilder`].

```rust
fn builder() -> FleetBuilder
```

### `Fleet::from_state`

Restores a fleet from a previously saved [`State`], for a gateway that persists its
fleet across restarts (save what [`snapshot`](StateSource::snapshot) returns, reload
it here on boot).

**Arguments**

* `state` - the fleet structure and last readings to restore.

**Returns**

A fleet holding the restored state.

```rust
fn from_state(state: State) -> Self
```

### `Fleet::report_reading`

Pushes a fresh reading for a sensor, appending it to the sensor's history.

**Arguments**

* `group` - the sensor's group id.
* `sensor` - the sensor id.
* `reading` - the new reading (the caller sets its status and band).

```rust
fn report_reading(&self, group: &str, sensor: &str, reading: Reading)
```

### `Fleet::report_event`

Records a recent event for a sensor, newest first.

**Arguments**

* `group` - the sensor's group id.
* `sensor` - the sensor id.
* `event` - the event to record.

```rust
fn report_event(&self, group: &str, sensor: &str, event: EventRecord)
```

### `Fleet::report_link`

Updates a group's link status (kind, signal strength, online).

**Arguments**

* `group` - the group id.
* `link` - the new link status.

```rust
fn report_link(&self, group: &str, link: Link)
```

### `Fleet::report_power`

Updates a sensor's power mode and battery state of charge.

**Arguments**

* `group` - the sensor's group id.
* `sensor` - the sensor id.
* `mode` - the work cadence the sensor's node is running at.
* `battery` - the state of charge in `[0.0, 1.0]`, or `None` if it has no battery.

```rust
fn report_power(&self, group: &str, sensor: &str, mode: Mode, battery: Option <f32>)
```

### `Fleet::take_commands`

Drains the control commands queued since the last call, for the project to apply to
real hardware and persist, then reflect back through the report methods.

**Returns**

The commands accepted since the last drain, in order.

```rust
fn take_commands(&self) -> Vec <Command>
```

### `Fleet::add_group`

Adds a group to an organization at runtime, so a gateway can surface a node the moment
it is discovered (a LoRa join, a new mesh neighbour). A no-op if the org is unknown.

**Arguments**

* `org` - the organization id to add the group to.
* `group` - the group to add.

```rust
fn add_group(&self, org: &str, group: Group)
```

### `Fleet::add_sensor`

Adds a sensor to a group at runtime, for a newly discovered sensor. A no-op if the
group is unknown.

**Arguments**

* `group` - the group id to add the sensor to.
* `sensor` - the sensor to add.

```rust
fn add_sensor(&self, group: &str, sensor: Sensor)
```

### `Fleet::remove_group`

Removes a group by id at runtime, for a node that has gone away.

**Arguments**

* `id` - the group id to remove.

```rust
fn remove_group(&self, id: &str)
```

### `Fleet::remove_sensor`

Removes a sensor by its `"groupId/sensorId"` path at runtime.

**Arguments**

* `target` - the `"groupId/sensorId"` path to remove.

```rust
fn remove_sensor(&self, target: &str)
```

## struct `FleetBuilder`

Builds a fleet's initial structure: organizations, their groups, and each group's
sensors. Parents are referenced by id, so add an org before its groups and a group
before its sensors.

### `FleetBuilder::org`

Adds an organization.

**Arguments**

* `id` - the stable organization id.
* `name` - the human-readable name.

**Returns**

The builder, for chaining.

```rust
fn org(mut self, id: impl Into <String>, name: impl Into <String>) -> Self
```

### `FleetBuilder::group`

Adds a group to an organization.

**Arguments**

* `org` - the id of the organization to add the group to.
* `id` - the stable group id.
* `name` - the human-readable name.
* `kind` - the link the group reports over.

**Returns**

The builder, for chaining.

```rust
fn group(mut self, org: &str, id: impl Into <String>, name: impl Into <String>, kind: LinkKind,) -> Self
```

### `FleetBuilder::sensor`

Adds a sensor to a group.

**Arguments**

* `group` - the id of the group to add the sensor to.
* `sensor` - the sensor (build its reading with [`Reading`] and [`Sensor::new`]).

**Returns**

The builder, for chaining.

```rust
fn sensor(mut self, group: &str, sensor: Sensor) -> Self
```

### `FleetBuilder::build`

Finishes building, returning a [`Fleet`] ready to serve and report into.

**Returns**

The assembled fleet.

```rust
fn build(mut self) -> Fleet
```

