# state

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

The language-neutral fleet snapshot a gateway serves to its dashboard.

A dashboard often watches more than one node: a clinic with several cold-chain
fridges, a co-op with many silos, a watershed of river gauges. So the snapshot is a
fleet - organizations, each with sensor groups, each group on its own link and
holding its own sensors. Everything human-facing travels as stable keys, stable
codes, raw values, and canonical units, identical in every locale; the page renders
the words and the formatting at the surface.

## enum `Status`

The health of a sensor, group, or the whole fleet, the basis of the glance-first UI.

- `Ok` - Everything is within its safe band. Ordered least urgent, so the derived ordering makes [`Status::worst`] a simple `max`.
- `Warn` - Something needs attention but is not yet critical.
- `Alarm` - A safety threshold has been crossed and action is needed now.

### `Status::worst`

Returns the more urgent of two statuses.

**Arguments**

* `other` - the status to compare against.

**Returns**

The most urgent of the two.

```rust
fn worst(self, other: Status) -> Status
```

## enum `Trend`

The direction a reading is moving, drawn as a trend arrow.

- `Rising` - The reading is rising.
- `Steady` - The reading is steady.
- `Falling` - The reading is falling.

## struct `Reading`

A single measured value, named by a stable key and a canonical unit.

Fields:

- `key: String` - A stable, language-neutral element key, such as `"soil_moisture"`.
- `value: f32` - The raw measured value, in the canonical unit.
- `unit: String` - The canonical unit name, such as `"percent"`, `"celsius"`, or `"volt"`.
- `status: Status` - The health of this reading on its own.
- `band: Option <[f32 ; 2]>` - The safe band `[low, high]` in the same unit, drawn as the gauge's safe zone.
- `trend: Option <Trend>` - Which way the reading is moving, if known.
- `state: Option <String>` - A discrete state code for a non-numeric reading, such as `"state.open"` for a valve or `"pump.nominal"` for a pump, which the page renders as a labelled chip. Numeric readings leave this `None`.
- `actions: Option <Vec <String>>` - The discrete actions this reading can be commanded to, such as `["open", "closed"]` for a valve. Present only on a controllable actuator; a read-only sensor leaves this `None`, and the page shows control only when it is set.

### `Reading::new`

Creates a reading in good standing with no band or trend.

**Arguments**

* `key` - the stable element key.
* `value` - the raw measured value.
* `unit` - the canonical unit name.

**Returns**

A [`Status::Ok`] reading carrying just the value and unit.

```rust
fn new(key: impl Into <String>, value: f32, unit: impl Into <String>) -> Self
```

### `Reading::with_status`

Sets the reading's health.

**Arguments**

* `status` - the health to record.

**Returns**

The reading, for chaining.

```rust
fn with_status(mut self, status: Status) -> Self
```

### `Reading::with_band`

Sets the safe band drawn as the gauge's safe zone.

**Arguments**

* `low` - the bottom of the safe band.
* `high` - the top of the safe band.

**Returns**

The reading, for chaining.

```rust
fn with_band(mut self, low: f32, high: f32) -> Self
```

### `Reading::with_trend`

Sets the reading's trend arrow.

**Arguments**

* `trend` - which way the reading is moving.

**Returns**

The reading, for chaining.

```rust
fn with_trend(mut self, trend: Trend) -> Self
```

### `Reading::with_state`

Sets a discrete state code for a non-numeric reading, rendered as a chip.

**Arguments**

* `state` - the stable state code, such as `"state.open"`.

**Returns**

The reading, for chaining.

```rust
fn with_state(mut self, state: impl Into <String>) -> Self
```

### `Reading::with_actions`

Marks the reading as a controllable actuator with the given discrete actions.

**Arguments**

* `actions` - the action codes a client may command, such as `["open", "closed"]`.

**Returns**

The reading, for chaining.

```rust
fn with_actions(mut self, actions: impl IntoIterator <Item = impl Into <String>>) -> Self
```

## enum `EventLevel`

The severity of a telemetry event, mirrored onto the wire as a stable string.

- `Trace` - Fine-grained detail.
- `Debug` - Diagnostic detail.
- `Info` - A normal, noteworthy event.
- `Warn` - Something unexpected the node recovered from.
- `Error` - A failure that needs attention.

## struct `EventRecord`

One recent telemetry event, carried as a stable code the page localizes.

Fields:

- `level: EventLevel` - The event's severity.
- `code: String` - The stable, short event code, such as `"battery.low"` or `"link.lost"`.
- `value: Option <f32>` - An optional measurement that came with the event.
- `age_secs: Option <u64>` - How many seconds ago the event happened, for a relative "x ago" display.

### `EventRecord::from_event`

Builds a record from a telemetry [`Event`] and how long ago it happened.

**Arguments**

* `event` - the telemetry event to mirror onto the wire.
* `age_secs` - how many seconds ago it happened, or `None` if unknown.

**Returns**

The serializable event record.

```rust
fn from_event(event: &Event, age_secs: Option <u64>) -> Self
```

## enum `Mode`

The work cadence a node is running at, mirrored from [`PowerMode`].

- `Active` - Healthy charge: the normal cadence.
- `Saver` - Low charge: a stretched cadence to conserve.
- `Critical` - Critically low charge: the bare minimum to survive.

## enum `LinkKind`

The kind of link a group reports over, shown as a labelled service before the bars.

- `Lora` - Long-range, low-power radio.
- `Wifi` - Local WiFi.
- `Cellular` - A cellular modem (LTE-M, 2G/4G, or similar).
- `NbIot` - A narrowband-IoT cellular link, common for low-power field clinics.
- `Satellite` - A satellite uplink.
- `Ethernet` - Wired Ethernet.
- `Mesh` - A multi-hop radio mesh.

## struct `Link`

A group's connectivity: what it talks over, how strong it is, and whether it is up.

Fields:

- `kind: LinkKind` - The kind of link.
- `strength: u8` - Signal strength as a bar count in `0..=4`.
- `online: bool` - Whether the group currently has any uplink at all.

## struct `Sensor`

A single sensor: its current reading, recent history, power, and recent events.

Fields:

- `id: String` - A stable, human-readable sensor identifier, such as `"fridge-1"`.
- `reading: Reading` - The sensor's current reading.
- `battery: Option <f32>` - The sensor's battery state of charge in `[0.0, 1.0]`, if it has one.
- `mode: Mode` - The work cadence the sensor's node is running at.
- `history: Vec <f32>` - Recent values of the reading, oldest first, for a sparkline and min/max.
- `events: Vec <EventRecord>` - The most recent telemetry events for this sensor, newest first.

### `Sensor::new`

Creates a sensor with an id and current reading, no battery, history, or events yet.

**Arguments**

* `id` - the stable sensor identifier.
* `reading` - the sensor's current reading.

**Returns**

An [`Mode::Active`] sensor carrying just the reading.

```rust
fn new(id: impl Into <String>, reading: Reading) -> Self
```

## struct `Group`

A group of sensors sharing one node and one link, such as a clinic's fridges.

Fields:

- `id: String` - A stable group identifier.
- `name: String` - A human-readable group name, such as `"Kano cold chain"`.
- `link: Link` - The group's link.
- `status: Status` - The group's overall health, the worst of its sensors.
- `sensors: Vec <Sensor>` - The sensors in the group.

### `Group::recompute_status`

Recomputes the group's [`status`](Group::status) from its sensors and events.

**Returns**

The group's overall status, also stored back into [`status`](Group::status).

```rust
fn recompute_status(&mut self) -> Status
```

## struct `Org`

An organization, such as a health authority or a farming co-op.

Fields:

- `id: String` - A stable organization identifier.
- `name: String` - A human-readable organization name.
- `groups: Vec <Group>` - The sensor groups belonging to the organization.

## struct `State`

The complete language-neutral fleet snapshot served at `GET /state`.

This is the single source the dashboard renders from. It is byte-identical in
every locale; the page supplies all words and formatting.

Fields:

- `orgs: Vec <Org>` - The organizations in the fleet.
- `status: Status` - The fleet's overall health, the worst across every group.
- `uptime_secs: Option <u64>` - Seconds the gateway has been running, if tracked.
- `demo: bool` - Whether this snapshot comes from the hardware-free demo, not a real device. The page shows demo-only affordances (the scenario switcher) only when this is set; a real device omits it.

### `State::recompute_status`

Recomputes every group's status and the fleet's overall status.

**Returns**

The fleet's overall status, also stored back into [`status`](State::status).

```rust
fn recompute_status(&mut self) -> Status
```

### `State::to_json`

Serializes the snapshot to the compact JSON served at `GET /state`.

**Returns**

The JSON text of the snapshot.

**Errors**

Returns a [`serde_json::Error`] if the snapshot cannot be serialized, which in
practice only happens on a non-finite float.

```rust
fn to_json(&self) -> Result <String, serde_json::Error>
```

### `State::from_json`

Parses a snapshot from its JSON form, for restoring a persisted fleet on boot.

**Arguments**

* `json` - the JSON text of a previously serialized snapshot.

**Returns**

The parsed [`State`].

**Errors**

Returns a [`serde_json::Error`] if the JSON is malformed or does not match the shape.

```rust
fn from_json(json: &str) -> Result <Self, serde_json::Error>
```

