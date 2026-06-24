# pamoja-kit::median

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

A rolling median filter for rejecting spikes.

## struct `Median`

A median filter over the most recent `N` readings.

A single bad sample - a spike from electrical noise or a flaky contact - drags a mean or
an exponential average off course, because it is blended into the result. The median
ignores it: one outlier cannot move the middle value of a sorted window. That makes a
[`Median`] the right filter when the noise is occasional spikes rather than steady
jitter; for steady jitter reach for [`Smoother`](crate::Smoother) instead. Keep the
window small and odd (3, 5, 7) so there is a single middle reading; with an even `N` the
median is the average of the two middle readings.

**Examples**

```
use pamoja_kit::Median;

let mut filtered = Median::<5>::new();
// A lone spike among steady readings is rejected.
for reading in [10.0, 10.0, 99.0, 10.0, 10.0] {
    filtered.update(reading);
}
assert_eq!(filtered.median(), Some(10.0));
```

### `Median <N>::new`

Creates an empty median filter over a window of `N` readings.

**Returns**

A filter holding no readings yet.

```rust
fn new() -> Self
```

### `Median <N>::update`

Adds a reading and returns the median of the current window.

**Arguments**

* `reading` - the latest raw reading.

**Returns**

The median of the readings now in the window. With a zero-length window (`N` is `0`)
the reading passes through unchanged.

```rust
fn update(&mut self, reading: f32) -> f32
```

### `Median <N>::push`

Adds a reading to the window, evicting the oldest once it is full.

**Arguments**

* `reading` - the latest raw reading.

```rust
fn push(&mut self, reading: f32)
```

### `Median <N>::median`

Returns the median of the readings in the window, or [`None`] if it is empty.

**Returns**

The middle reading of the sorted window for an odd count, the average of the two
middle readings for an even count, or [`None`] before any reading.

```rust
fn median(&self) -> Option <f32>
```

### `Median <N>::len`

Returns the number of readings currently held, at most `N`.

```rust
fn len(&self) -> usize
```

### `Median <N>::is_empty`

Returns `true` if the filter holds no readings.

```rust
fn is_empty(&self) -> bool
```

