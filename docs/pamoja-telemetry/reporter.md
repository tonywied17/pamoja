# pamoja-telemetry::reporter

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

Recording events, filtering them for the link, and summarizing the counts.

## enum `LinkCost`

How costly the current link is, which sets how selective telemetry should be.

The cost maps to the level [`threshold`](LinkCost::threshold) a
[`Reporter`] should use: a free link ships everything, while an expensive or
absent link ships only what is worth its bytes.

- `Free` - A free or local link: ship all detail.
- `Metered` - A metered link: skip routine detail.
- `Expensive` - An expensive link, such as satellite: ship only warnings and errors.
- `Offline` - No link: hold back everything but errors worth buffering.

### `LinkCost::threshold`

Returns the level threshold this link cost calls for.

**Returns**

The minimum [`Level`] a reporter should ship at this link cost.

```rust
fn threshold(self) -> Level
```

## struct `Snapshot`

A point-in-time summary of a reporter's counters.

This is what a node ships periodically in place of the raw event stream: a few
integers that capture how many events occurred at each level and how many were
shipped versus dropped, cheap to send even on a metered link.

Fields:

- `by_level: [u32 ; LEVEL_COUNT]` - The number of events seen at each level, indexed by the level's order from [`Trace`](Level::Trace) to [`Error`](Level::Error).
- `emitted: u32` - How many events passed the filter and were shipped.
- `dropped: u32` - How many events were dropped by the filter.

## struct `Reporter`

Records telemetry events, ships the ones worth their bytes, and counts them all.

A reporter keeps a level threshold and forwards only events at or above it, while
counting every event it sees - dropped or not - so the aggregate picture survives
even when the link is too costly to ship detail. Call
[`adapt_to`](Reporter::adapt_to) as the link cost changes to raise or lower the
bar, and ship a [`snapshot`](Reporter::snapshot) of the counters periodically
instead of the full stream.

**Examples**

```
use pamoja_telemetry::{Event, Level, LinkCost, Reporter};

let mut reporter = Reporter::new(Level::Trace);

// On a metered link, routine debug events are dropped but a warning still ships.
reporter.adapt_to(LinkCost::Metered);
assert!(reporter.record(Event::debug("loop.tick")).is_none());
assert!(reporter.record(Event::warn("battery.low")).is_some());

// Both events were still counted.
assert_eq!(reporter.total(), 2);
assert_eq!(reporter.dropped(), 1);
```

### `Reporter::new`

Creates a reporter that ships events at or above `threshold`.

**Arguments**

* `threshold` - the minimum level to ship.

**Returns**

A reporter with empty counters.

```rust
fn new(threshold: Level) -> Self
```

### `Reporter::threshold`

Returns the current ship threshold.

**Returns**

The minimum level currently being shipped.

```rust
fn threshold(&self) -> Level
```

### `Reporter::set_threshold`

Sets the ship threshold directly.

**Arguments**

* `threshold` - the new minimum level to ship.

```rust
fn set_threshold(&mut self, threshold: Level)
```

### `Reporter::adapt_to`

Raises or lowers the threshold to match the current link cost.

**Arguments**

* `cost` - how costly the link currently is.

```rust
fn adapt_to(&mut self, cost: LinkCost)
```

### `Reporter::record`

Records an event, returning it to ship if it clears the threshold.

The event is counted whether or not it is shipped, so the aggregate counts
stay complete even while detail is held back.

**Arguments**

* `event` - the event to record.

**Returns**

`Some(event)` if it should be shipped, or `None` if it was dropped by the
threshold.

```rust
fn record(&mut self, event: Event) -> Option <Event>
```

### `Reporter::count`

Returns how many events have been seen at `level`, shipped or not.

**Arguments**

* `level` - the level to count.

**Returns**

The number of events recorded at that level.

```rust
fn count(&self, level: Level) -> u32
```

### `Reporter::total`

Returns the total number of events seen across all levels.

**Returns**

The total count.

```rust
fn total(&self) -> u32
```

### `Reporter::emitted`

Returns how many events passed the threshold and were shipped.

**Returns**

The emitted count.

```rust
fn emitted(&self) -> u32
```

### `Reporter::dropped`

Returns how many events were dropped by the threshold.

**Returns**

The dropped count.

```rust
fn dropped(&self) -> u32
```

### `Reporter::snapshot`

Returns a snapshot of the counters to ship in place of the raw stream.

**Returns**

A [`Snapshot`] of the per-level counts and the emitted and dropped totals.

```rust
fn snapshot(&self) -> Snapshot
```

