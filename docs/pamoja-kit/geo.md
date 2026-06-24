# pamoja-kit::geo

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

Working with positions on the Earth: distance and staying inside an area.

## struct `Coordinate`

A position on the Earth, in decimal degrees.

Latitude and longitude are kept as `f64` because a GPS fix needs more precision
than `f32` can hold: rounding a coordinate to `f32` can move it tens of metres.

**Examples**

Great-circle distance between two cities, in kilometres:

```
use pamoja_kit::Coordinate;

let nairobi = Coordinate::new(-1.2921, 36.8219);
let mombasa = Coordinate::new(-4.0435, 39.6682);
let km = nairobi.distance_to(mombasa) / 1000.0;
assert!((km - 440.0).abs() < 10.0); // about 440 km apart
```

Fields:

- `latitude: f64` - Degrees north of the equator, in `[-90.0, 90.0]`.
- `longitude: f64` - Degrees east of the prime meridian, in `[-180.0, 180.0]`.

### `Coordinate::new`

Creates a coordinate from a latitude and longitude in decimal degrees.

**Arguments**

* `latitude` - degrees north of the equator.
* `longitude` - degrees east of the prime meridian.

**Returns**

The coordinate.

```rust
fn new(latitude: f64, longitude: f64) -> Self
```

### `Coordinate::distance_to`

Returns the distance to another coordinate in metres.

This is the great-circle distance: the shortest path over the surface of a
spherical Earth. The technique one layer down is the haversine formula, which
stays numerically stable for the short distances a field deployment cares
about, down to points a few metres apart.

**Arguments**

* `other` - the coordinate to measure to.

**Returns**

The distance in metres, always zero or positive.

```rust
fn distance_to(&self, other: Coordinate) -> f64
```

### `Coordinate::bearing_to`

Returns the initial bearing to another coordinate, in degrees clockwise from north.

This is the forward azimuth of the great-circle path: the compass heading to set off
on to reach `other` by the shortest route. Because a great circle curves, the bearing
changes along the way; this is the heading at the start. The result is normalised to
`[0.0, 360.0)`, with 0 north, 90 east, 180 south, and 270 west.

**Arguments**

* `other` - the coordinate to head toward.

**Returns**

The initial bearing in degrees, in `[0.0, 360.0)`. When both points are the same the
result is `0.0`.

```rust
fn bearing_to(&self, other: Coordinate) -> f64
```

## enum `Boundary`

Where a fix sits relative to a [`Geofence`], including the moment it crosses.

- `Inside` - The fix is inside the fence and was inside before, or is the first fix inside.
- `Outside` - The fix is outside the fence and was outside before, or is the first fix outside.
- `Exited` - The fix just crossed from inside to outside: the moment to raise a breach alert.
- `Entered` - The fix just crossed from outside back inside.

## struct `Geofence`

Keeping a tracked point inside an area, and noticing when it leaves.

This is the primitive behind "tell me when it leaves the safe zone": a collared
animal straying from its pasture, an asset moving off-site, or a drone crossing
its allowed boundary. A fence is a centre and a radius; feeding it successive
fixes reports whether each is [`Inside`](Boundary::Inside) or
[`Outside`](Boundary::Outside) and, crucially, the single fix that
[`Exited`](Boundary::Exited) or [`Entered`](Boundary::Entered), so an alert fires
once on the crossing rather than on every fix while away.

**Examples**

```
use pamoja_kit::{Boundary, Coordinate, Geofence};

// A 50 m pen around the waterpoint; the collar fix then wanders out.
let mut pen = Geofence::new(Coordinate::new(-1.2921, 36.8219), 50.0);
assert_eq!(pen.update(Coordinate::new(-1.2921, 36.8219)), Boundary::Inside);
assert_eq!(pen.update(Coordinate::new(-1.2930, 36.8219)), Boundary::Exited);
```

### `Geofence::new`

Creates a fence of `radius_m` metres around `center`.

**Arguments**

* `center` - the middle of the safe area.
* `radius_m` - the radius of the safe area in metres; its magnitude is used.

**Returns**

A fence that has not yet seen a fix.

```rust
fn new(center: Coordinate, radius_m: f64) -> Self
```

### `Geofence::contains`

Returns whether a point lies within the fence.

**Arguments**

* `point` - the coordinate to test.

**Returns**

`true` if `point` is on or inside the fence boundary.

```rust
fn contains(&self, point: Coordinate) -> bool
```

### `Geofence::update`

Records a fix and reports its position relative to the fence.

**Arguments**

* `point` - the latest fix.

**Returns**

[`Boundary::Entered`] or [`Boundary::Exited`] on the fix that crosses the
boundary, otherwise [`Boundary::Inside`] or [`Boundary::Outside`].

```rust
fn update(&mut self, point: Coordinate) -> Boundary
```

