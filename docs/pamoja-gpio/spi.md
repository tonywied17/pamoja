# pamoja-gpio::spi

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

SPI clock modes and bit order.

SPI has no addressing and no framing of its own: a transfer is just bytes clocked in
and out at the same time. What a controller and a peripheral must agree on is when the
clock idles and which clock edge samples data - the two bits CPOL (clock polarity) and
CPHA (clock phase) - and whether each byte travels most- or least-significant bit
first. Datasheets quote the CPOL/CPHA pair as a single mode number from 0 to 3, and the
commonest cause of a dead SPI link is a transposed pair or the wrong mode. This module
makes the mode a checked value rather than two loose booleans a caller can swap.

## enum `Mode`

An SPI clock mode: the `(CPOL, CPHA)` pair a controller and peripheral must share.

The mode number is `(CPOL << 1) | CPHA`, so the four modes are:

| Mode | CPOL | CPHA | Clock idles | Data sampled on |
| --- | --- | --- | --- | --- |
| 0 | 0 | 0 | low | leading edge (rising) |
| 1 | 0 | 1 | low | trailing edge (falling) |
| 2 | 1 | 0 | high | leading edge (falling) |
| 3 | 1 | 1 | high | trailing edge (rising) |

**Examples**

```
use pamoja_gpio::spi::Mode;

// An SD card and most LoRa radios use mode 0.
assert_eq!(Mode::Mode0.number(), 0);
assert_eq!(Mode::Mode0.cpol_cpha(), (false, false));
assert_eq!(Mode::from_number(3), Some(Mode::Mode3));
assert_eq!(Mode::from_cpol_cpha(true, false), Mode::Mode2);
```

- `Mode0` - CPOL 0, CPHA 0: clock idles low, data sampled on the rising (leading) edge.
- `Mode1` - CPOL 0, CPHA 1: clock idles low, data sampled on the falling (trailing) edge.
- `Mode2` - CPOL 1, CPHA 0: clock idles high, data sampled on the falling (leading) edge.
- `Mode3` - CPOL 1, CPHA 1: clock idles high, data sampled on the rising (trailing) edge.

### `Mode::number`

Returns the mode number, `0..=3`, as datasheets quote it.

**Returns**

The number `(CPOL << 1) | CPHA`.

```rust
fn number(self) -> u8
```

### `Mode::from_number`

Returns the mode a number names, if it is in range.

**Arguments**

* `number` - a mode number.

**Returns**

The matching [`Mode`], or [`None`] if `number` is above `3`.

```rust
fn from_number(number: u8) -> Option <Mode>
```

### `Mode::cpol_cpha`

Returns the `(CPOL, CPHA)` pair for this mode.

**Returns**

`(clock idles high, data sampled on the trailing edge)`.

```rust
fn cpol_cpha(self) ->(bool, bool)
```

### `Mode::from_cpol_cpha`

Returns the mode for a `(CPOL, CPHA)` pair.

**Arguments**

* `cpol` - clock polarity: `true` if the clock idles high.
* `cpha` - clock phase: `true` if data is sampled on the trailing edge.

**Returns**

The matching [`Mode`]. Every pair maps to a mode, so this never fails.

```rust
fn from_cpol_cpha(cpol: bool, cpha: bool) -> Mode
```

### `Mode::clock_idles_high`

Returns `true` if the clock idles high (CPOL = 1), which is modes 2 and 3.

```rust
fn clock_idles_high(self) -> bool
```

### `Mode::samples_on_trailing_edge`

Returns `true` if data is sampled on the trailing clock edge (CPHA = 1), which is
modes 1 and 3.

```rust
fn samples_on_trailing_edge(self) -> bool
```

## enum `BitOrder`

The order bits travel within each SPI byte.

- `MsbFirst` - Most-significant bit first. The common default for nearly every SPI peripheral.
- `LsbFirst` - Least-significant bit first.

