# pamoja-kit::trend

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

Measuring whether a value is trending up or down.

## struct `Trend`

Tracks the linear trend of the most recent `N` readings.

Is a value rising or falling, and how fast? A [`Trend`] fits a least-squares straight
line to the last `N` readings, taken as evenly spaced in time, and reports its slope in
units per sample: positive when rising, negative when falling, near zero when flat.
Because the fit uses the whole window, a single noisy reading does not masquerade as a
trend the way a bare difference between two samples can. The slope is the ordinary
least-squares estimate, the sum of `(x - mean_x) * (y - mean_y)` over the sum of
`(x - mean_x)` squared, with `x` taken as the sample index 0, 1, 2, and so on.

**Examples**

```
use pamoja_kit::Trend;

// A tank level falling two units per reading.
let mut level = Trend::<4>::new();
for reading in [40.0, 38.0, 36.0, 34.0] {
    level.push(reading);
}
assert!((level.slope().unwrap() + 2.0).abs() < 1e-4);
```

### `Trend <N>::new`

Creates an empty trend tracker over a window of `N` readings.

**Returns**

A tracker holding no readings yet.

```rust
fn new() -> Self
```

### `Trend <N>::push`

Adds a reading, evicting the oldest once the window is full.

**Arguments**

* `reading` - the latest reading, taken one sample interval after the previous one.

```rust
fn push(&mut self, reading: f32)
```

### `Trend <N>::slope`

Returns the trend slope in units per sample, or [`None`] with fewer than two readings.

**Returns**

The least-squares slope of the window: positive when rising, negative when falling.
[`None`] until at least two readings are present, since a single point has no slope.

```rust
fn slope(&self) -> Option <f32>
```

### `Trend <N>::len`

Returns the number of readings currently held, at most `N`.

```rust
fn len(&self) -> usize
```

### `Trend <N>::is_empty`

Returns `true` if the tracker holds no readings.

```rust
fn is_empty(&self) -> bool
```

