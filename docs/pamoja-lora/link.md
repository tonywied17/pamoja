# pamoja-lora::link

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

LoRa link settings and the time-on-air they imply.

## struct `LinkSettings`

The radio settings of a LoRa link, enough to compute its time-on-air.

A LoRa transmission's duration is fixed by the spreading factor, the bandwidth,
the coding rate, and the frame options, not by the data itself beyond its length.
This struct gathers those settings and computes the two numbers a long-range
deployment lives by: the [`airtime`](LinkSettings::airtime_us) of a payload, and
the [`off time`](LinkSettings::min_off_time_us) a duty-cycle limit then forces
before the next transmission.

A higher spreading factor reaches much further but spends far longer on air, so the
same payload that takes tens of milliseconds at SF7 can take most of a second at
SF12, with a correspondingly longer mandatory silence. The arithmetic is exact and
integer-only, so it runs on the smallest node.

**Examples**

```
use pamoja_lora::LinkSettings;

// The default European long-range setup: SF12, 125 kHz, coding rate 4/5.
let link = LinkSettings::new(12, 125_000);

// A 10-byte payload takes just under a second on air at SF12.
assert_eq!(link.airtime_us(10), 991_232);
```

### `LinkSettings::new`

Creates link settings from a spreading factor and bandwidth, with LoRa defaults.

The defaults are coding rate 4/5, an 8-symbol preamble, an explicit header, and
CRC on, matching a typical uplink.

**Arguments**

* `spreading_factor` - the spreading factor; clamped to the LoRa range 7 to 12.
* `bandwidth_hz` - the channel bandwidth in hertz, such as `125_000`.

**Returns**

The link settings.

```rust
fn new(spreading_factor: u8, bandwidth_hz: u32) -> Self
```

### `LinkSettings::with_coding_rate`

Sets the coding rate by its denominator, from 4/5 to 4/8.

**Arguments**

* `denominator` - the coding-rate denominator, clamped to 5 to 8 for 4/5 to 4/8.

**Returns**

The updated settings, for chaining.

```rust
fn with_coding_rate(mut self, denominator: u8) -> Self
```

### `LinkSettings::with_preamble`

Sets the number of preamble symbols.

**Arguments**

* `symbols` - the preamble length in symbols; the LoRa default is 8.

**Returns**

The updated settings, for chaining.

```rust
fn with_preamble(mut self, symbols: u16) -> Self
```

### `LinkSettings::implicit_header`

Uses an implicit header, which omits the header symbols from each frame.

**Returns**

The updated settings, for chaining.

```rust
fn implicit_header(mut self) -> Self
```

### `LinkSettings::without_crc`

Turns the frame CRC off.

**Returns**

The updated settings, for chaining.

```rust
fn without_crc(mut self) -> Self
```

### `LinkSettings::spreading_factor`

Returns the spreading factor.

**Returns**

The spreading factor, from 7 to 12.

```rust
fn spreading_factor(&self) -> u8
```

### `LinkSettings::bandwidth_hz`

Returns the channel bandwidth in hertz.

**Returns**

The bandwidth in hertz.

```rust
fn bandwidth_hz(&self) -> u32
```

### `LinkSettings::symbol_time_us`

Returns the duration of one symbol in microseconds.

**Returns**

The symbol time, `2^spreading_factor / bandwidth`, in microseconds.

```rust
fn symbol_time_us(&self) -> u64
```

### `LinkSettings::airtime_us`

Returns the time on air of a payload in microseconds.

This is the channel occupancy the transmission costs: how long the radio holds
the air, which sets both the duty-cycle budget and a large part of the energy
the transmission spends.

**Arguments**

* `payload_len` - the payload length in bytes.

**Returns**

The time on air in microseconds.

```rust
fn airtime_us(&self, payload_len: usize) -> u64
```

### `LinkSettings::min_off_time_us`

Returns the minimum silence after a transmission to honor a duty-cycle limit.

A duty-cycle limit caps the fraction of time a node may transmit, so after a
transmission of a given airtime the node must stay quiet for long enough that
the airtime is no more than that fraction of the whole cycle.

**Arguments**

* `payload_len` - the payload length in bytes.
* `duty_cycle_permille` - the duty-cycle limit in parts per thousand, so `10`
  is 1%.

**Returns**

The required off time in microseconds, or [`u64::MAX`] if the limit is zero.

```rust
fn min_off_time_us(&self, payload_len: usize, duty_cycle_permille: u32) -> u64
```

