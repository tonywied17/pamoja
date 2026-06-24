# pamoja-telemetry::event

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

Telemetry events: a severity level, a stable code, and an optional value.

## enum `Level`

The severity of a telemetry event, ordered from most verbose to most urgent.

[`Trace`](Level::Trace) is the least urgent and [`Error`](Level::Error) the most,
so a [`Reporter`](crate::Reporter) ships an event when its level is at or above the
current threshold and drops it otherwise.

- `Trace` - Fine-grained detail, useful only when chasing a specific problem.
- `Debug` - Diagnostic detail for development.
- `Info` - A normal, noteworthy event.
- `Warn` - Something unexpected that the node recovered from.
- `Error` - A failure that needs attention.

## struct `Event`

A structured telemetry event.

An event pairs a [`Level`] with a stable, short `code` - a label such as
`"battery.low"` or `"link.lost"` rather than a free-form message - so events stay
tiny, group cleanly into counts, and need no allocation. An optional `value`
carries an associated measurement, such as the battery level that triggered it.

Fields:

- `level: Level` - The event's severity.
- `code: &'static str` - A stable, short identifier for what happened.
- `value: Option <f32>` - An optional measurement associated with the event.

### `Event::new`

Creates an event at `level` with the given code and no value.

**Arguments**

* `level` - the event's severity.
* `code` - a stable, short identifier for the event.

**Returns**

The event.

```rust
fn new(level: Level, code: &'static str) -> Self
```

### `Event::trace`

Creates a [`Level::Trace`] event.

**Arguments**

* `code` - a stable, short identifier for the event.

**Returns**

The event.

```rust
fn trace(code: &'static str) -> Self
```

### `Event::debug`

Creates a [`Level::Debug`] event.

**Arguments**

* `code` - a stable, short identifier for the event.

**Returns**

The event.

```rust
fn debug(code: &'static str) -> Self
```

### `Event::info`

Creates a [`Level::Info`] event.

**Arguments**

* `code` - a stable, short identifier for the event.

**Returns**

The event.

```rust
fn info(code: &'static str) -> Self
```

### `Event::warn`

Creates a [`Level::Warn`] event.

**Arguments**

* `code` - a stable, short identifier for the event.

**Returns**

The event.

```rust
fn warn(code: &'static str) -> Self
```

### `Event::error`

Creates a [`Level::Error`] event.

**Arguments**

* `code` - a stable, short identifier for the event.

**Returns**

The event.

```rust
fn error(code: &'static str) -> Self
```

### `Event::with_value`

Attaches a measurement to the event.

**Arguments**

* `value` - the measurement to associate with the event.

**Returns**

The event, for chaining.

```rust
fn with_value(mut self, value: f32) -> Self
```

