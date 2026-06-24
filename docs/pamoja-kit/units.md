# pamoja-kit::units

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

Converting readings between real-world units.

Cheap environmental sensors report whatever unit their datasheet chose - a BME280
gives pressure in pascals, a thermocouple in degrees Celsius - while the person
reading a dashboard thinks in another. These are the exact, named conversions for the
units the cookbook uses most, so a conversion is a call with an obvious name rather
than a magic constant copied into application code.

## fn `celsius_to_fahrenheit`

Converts a temperature from degrees Celsius to degrees Fahrenheit.

**Arguments**

* `celsius` - a temperature in degrees Celsius.

**Returns**

The temperature in degrees Fahrenheit, `celsius * 9 / 5 + 32`.

```rust
fn celsius_to_fahrenheit(celsius: f32) -> f32
```

## fn `fahrenheit_to_celsius`

Converts a temperature from degrees Fahrenheit to degrees Celsius.

**Arguments**

* `fahrenheit` - a temperature in degrees Fahrenheit.

**Returns**

The temperature in degrees Celsius, `(fahrenheit - 32) * 5 / 9`.

```rust
fn fahrenheit_to_celsius(fahrenheit: f32) -> f32
```

## fn `celsius_to_kelvin`

Converts a temperature from degrees Celsius to kelvin.

**Arguments**

* `celsius` - a temperature in degrees Celsius.

**Returns**

The temperature in kelvin, `celsius + 273.15`.

```rust
fn celsius_to_kelvin(celsius: f32) -> f32
```

## fn `kelvin_to_celsius`

Converts a temperature from kelvin to degrees Celsius.

**Arguments**

* `kelvin` - a temperature in kelvin.

**Returns**

The temperature in degrees Celsius, `kelvin - 273.15`.

```rust
fn kelvin_to_celsius(kelvin: f32) -> f32
```

## fn `pascals_to_hectopascals`

Converts a pressure from pascals to hectopascals (millibars).

**Arguments**

* `pascals` - a pressure in pascals.

**Returns**

The pressure in hectopascals, the unit weather reports use, `pascals / 100`.

```rust
fn pascals_to_hectopascals(pascals: f32) -> f32
```

## fn `hectopascals_to_pascals`

Converts a pressure from hectopascals (millibars) to pascals.

**Arguments**

* `hectopascals` - a pressure in hectopascals.

**Returns**

The pressure in pascals, `hectopascals * 100`.

```rust
fn hectopascals_to_pascals(hectopascals: f32) -> f32
```

## fn `pascals_to_kilopascals`

Converts a pressure from pascals to kilopascals.

**Arguments**

* `pascals` - a pressure in pascals.

**Returns**

The pressure in kilopascals, `pascals / 1000`.

```rust
fn pascals_to_kilopascals(pascals: f32) -> f32
```

## fn `kilopascals_to_pascals`

Converts a pressure from kilopascals to pascals.

**Arguments**

* `kilopascals` - a pressure in kilopascals.

**Returns**

The pressure in pascals, `kilopascals * 1000`.

```rust
fn kilopascals_to_pascals(kilopascals: f32) -> f32
```

## fn `pascals_to_psi`

Converts a pressure from pascals to pounds per square inch.

**Arguments**

* `pascals` - a pressure in pascals.

**Returns**

The pressure in psi, where one psi is 6894.7573 pascals.

```rust
fn pascals_to_psi(pascals: f32) -> f32
```

## fn `psi_to_pascals`

Converts a pressure from pounds per square inch to pascals.

**Arguments**

* `psi` - a pressure in pounds per square inch.

**Returns**

The pressure in pascals, where one psi is 6894.7573 pascals.

```rust
fn psi_to_pascals(psi: f32) -> f32
```

## fn `ratio_to_percent`

Converts a fraction in `0.0..=1.0` to a percentage.

**Arguments**

* `ratio` - a fraction, where `1.0` is the whole.

**Returns**

The equivalent percentage, `ratio * 100`.

```rust
fn ratio_to_percent(ratio: f32) -> f32
```

## fn `percent_to_ratio`

Converts a percentage to a fraction in `0.0..=1.0`.

**Arguments**

* `percent` - a percentage, where `100.0` is the whole.

**Returns**

The equivalent fraction, `percent / 100`.

```rust
fn percent_to_ratio(percent: f32) -> f32
```

