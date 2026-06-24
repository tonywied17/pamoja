# pamoja-kit::window

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

Keeping a rolling window of recent readings.

## struct `Window`

A fixed-capacity window over the most recent `N` readings.

Many field decisions look not at the latest reading but at the recent run of them: the
lowest battery voltage in the last minute, the average flow over the last ten samples,
how widely a tank level is bouncing. A [`Window`] keeps the last `N` readings in a ring
buffer - no allocation, so it runs on a microcontroller - and reports their spread. It
is the base the forecasting helpers build on.

The population [`variance`](Window::variance) is given directly; the standard deviation
is its square root, left to the caller so the type stays dependency-free. Capacity `N`
should be at least one; a zero-capacity window simply holds nothing.

**Examples**

```
use pamoja_kit::Window;

// Keep the last four tank-level readings and read their spread.
let mut levels = Window::<4>::new();
for reading in [40.0, 42.0, 38.0, 41.0] {
    levels.push(reading);
}
assert!(levels.is_full());
assert_eq!(levels.min(), Some(38.0));
assert_eq!(levels.max(), Some(42.0));
assert_eq!(levels.range(), Some(4.0));
assert_eq!(levels.latest(), Some(41.0));
```

### `Window <N>::new`

Creates an empty window with capacity `N`.

**Returns**

A window holding no readings yet.

```rust
fn new() -> Self
```

### `Window <N>::push`

Adds a reading, evicting the oldest once the window is full.

**Arguments**

* `reading` - the value to record.

```rust
fn push(&mut self, reading: f32)
```

### `Window <N>::len`

Returns the number of readings currently held, at most `N`.

```rust
fn len(&self) -> usize
```

### `Window <N>::is_empty`

Returns `true` if the window holds no readings.

```rust
fn is_empty(&self) -> bool
```

### `Window <N>::is_full`

Returns `true` if the window holds its full capacity of `N` readings.

```rust
fn is_full(&self) -> bool
```

### `Window <N>::capacity`

Returns the window's capacity, `N`.

```rust
fn capacity(&self) -> usize
```

### `Window <N>::latest`

Returns the most recent reading, or [`None`] if the window is empty.

```rust
fn latest(&self) -> Option <f32>
```

### `Window <N>::oldest`

Returns the oldest reading still held, or [`None`] if the window is empty.

```rust
fn oldest(&self) -> Option <f32>
```

### `Window <N>::min`

Returns the smallest reading in the window, or [`None`] if it is empty.

```rust
fn min(&self) -> Option <f32>
```

### `Window <N>::max`

Returns the largest reading in the window, or [`None`] if it is empty.

```rust
fn max(&self) -> Option <f32>
```

### `Window <N>::range`

Returns the spread (largest minus smallest), or [`None`] if the window is empty.

```rust
fn range(&self) -> Option <f32>
```

### `Window <N>::mean`

Returns the mean of the readings, or [`None`] if the window is empty.

```rust
fn mean(&self) -> Option <f32>
```

### `Window <N>::variance`

Returns the population variance of the readings, or [`None`] if the window is empty.

**Returns**

The mean squared deviation from the mean. The standard deviation is its square root.

```rust
fn variance(&self) -> Option <f32>
```

