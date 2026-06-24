# pamoja-kit::weather

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

Humidity-derived values: the dew point.

## fn `dew_point`

Computes the dew point from temperature and relative humidity (Magnus formula).

The dew point is the temperature to which air must cool for its moisture to begin to
condense; it is the practical signal behind condensation, fog, and frost. This uses the
Magnus-Tetens approximation with the WMO coefficients (b = 17.62, c = 243.12 C), accurate
from roughly -45 to 60 C: with `gamma = ln(rh / 100) + b * t / (c + t)`, the dew point is
`c * gamma / (b - gamma)`. A dew point at or below 0 C means any condensation forms as
frost, the basis of an overnight frost warning for a crop.

**Arguments**

* `celsius` - the air temperature in degrees Celsius.
* `humidity_percent` - the relative humidity in percent, in `(0, 100]`. A value at or
  below zero is treated as a tiny positive value so the logarithm stays defined.

**Returns**

The dew point in degrees Celsius.

**Examples**

```
use pamoja_kit::weather::dew_point;

// 20 C air at 50% relative humidity dews near 9.3 C.
assert!((dew_point(20.0, 50.0) - 9.3).abs() < 0.2);

// Saturated air: the dew point equals the temperature.
assert!((dew_point(15.0, 100.0) - 15.0).abs() < 1e-6);
```

```rust
fn dew_point(celsius: f64, humidity_percent: f64) -> f64
```

