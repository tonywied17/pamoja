# pamoja-kit::anomaly

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

Flagging a reading that departs from its recent history.

## struct `Anomaly`

Flags a reading that lies far from the recent norm.

"Tell me when something is off" needs a baseline: a reading is suspicious only relative to
what is usual. An [`Anomaly`] keeps a rolling window of recent readings and flags one that
sits more than a chosen number of standard deviations from their mean - the three-sigma
rule, the standard z-score test, with the threshold left to the caller (`3.0` is the
usual choice). It is dependency-free: instead of taking a square root it compares the
squared deviation against the squared threshold, which is the same test.

With a perfectly flat baseline the spread is zero, so any change at all reads as
anomalous; real sensor noise gives a non-zero baseline, where this is not an issue.

**Examples**

```
use pamoja_kit::Anomaly;

let mut watch = Anomaly::<8>::new(3.0);
// Establish a steady baseline.
for reading in [10.0, 10.2, 9.8, 10.1, 9.9, 10.0, 10.2, 9.8] {
    watch.check(reading);
}
assert!(!watch.check(10.1)); // close to the norm: fine
assert!(watch.check(20.0)); // a far jump: flagged
```

### `Anomaly <N>::new`

Creates a detector that flags readings beyond `sigmas` standard deviations.

**Arguments**

* `sigmas` - the threshold in standard deviations; `3.0` is the common three-sigma
  rule. Its magnitude is used.

**Returns**

A detector with an empty history.

```rust
fn new(sigmas: f32) -> Self
```

### `Anomaly <N>::check`

Tests a reading against the recent norm, then folds it into the history.

The reading is judged against the window of earlier readings, so the value being
tested does not inflate its own baseline. Until at least two readings have been seen
there is no spread to judge against, so nothing is flagged.

**Arguments**

* `reading` - the latest reading.

**Returns**

`true` if `reading` lies more than the configured standard deviations from the mean
of the recent window.

```rust
fn check(&mut self, reading: f32) -> bool
```

### `Anomaly <N>::len`

Returns the number of readings in the baseline window so far.

```rust
fn len(&self) -> usize
```

### `Anomaly <N>::is_empty`

Returns `true` if no readings have been recorded yet.

```rust
fn is_empty(&self) -> bool
```

