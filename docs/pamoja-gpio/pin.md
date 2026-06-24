# pamoja-gpio::pin

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

The GPIO pin model: levels, pull and drive configuration, interrupt edges, and active
polarity.

A GPIO pin is the simplest interface on a board: one line that is either high or low.
The logic that still has to be right is the meaning of that level. A button wired to
ground through a pull-up reads low when pressed; a relay board sold as "active low"
switches on when its input is driven low. Treating "pressed" or "on" as if it always
meant a high level is a classic inversion bug. This module carries the small set of
GPIO concepts so that mapping is written down once rather than scattered through call
sites.

## enum `Level`

The physical voltage level on a pin.

- `Low` - A low level, near ground.
- `High` - A high level, near the supply voltage.

### `Level::inverted`

Returns the opposite level.

```rust
fn inverted(self) -> Level
```

### `Level::is_high`

Returns `true` if this is [`High`](Level::High).

```rust
fn is_high(self) -> bool
```

### `Level::is_low`

Returns `true` if this is [`Low`](Level::Low).

```rust
fn is_low(self) -> bool
```

### `Level::from_bool`

Returns the level a boolean names.

**Arguments**

* `high` - `true` for [`High`](Level::High), `false` for [`Low`](Level::Low).

```rust
fn from_bool(high: bool) -> Level
```

## enum `Direction`

Whether a pin reads its line (input) or drives it (output).

- `Input` - The pin reads the level on its line.
- `Output` - The pin drives the level on its line.

## enum `Pull`

The internal pull resistor applied to an input pin.

A floating input drifts and reads noise, so a pin reading a switch needs a defined
resting level from a pull resistor (internal where the chip offers one, external
otherwise).

- `None` - No internal pull; the line floats unless something external holds it.
- `Up` - An internal pull-up holds the line high when nothing drives it.
- `Down` - An internal pull-down holds the line low when nothing drives it.

## enum `Drive`

How an output pin drives its two states.

- `PushPull` - Push-pull: the pin actively drives both high and low.
- `OpenDrain` - Open-drain: the pin actively drives low and floats when high, so an external pull-up sets the high level. This is what shared, multi-device lines like I2C use.

## enum `Edge`

The signal transition that triggers a pin interrupt.

- `Rising` - A low-to-high transition.
- `Falling` - A high-to-low transition.
- `Both` - Either transition.

### `Edge::triggered_by`

Returns `true` if a change from `from` to `to` is an edge this trigger fires on.

**Arguments**

* `from` - the level before the change.
* `to` - the level after the change.

**Returns**

`true` if the transition matches this trigger; `false` for the other direction or
for no change at all.

```rust
fn triggered_by(self, from: Level, to: Level) -> bool
```

## enum `Polarity`

Whether a signal is asserted by a high or a low physical level.

Active-low wiring is everywhere in cheap hardware: a button to ground with a pull-up
reads [`Level::Low`] when pressed, and many relay boards energise when their input is
driven low. This type maps between the logical idea of "asserted" and the physical
[`Level`] so the mapping lives in one place instead of in scattered inversions.

**Examples**

```
use pamoja_gpio::pin::{Level, Polarity};

// An active-low relay: asserting it (switching the relay on) drives the pin low.
let relay = Polarity::ActiveLow;
assert_eq!(relay.level(true), Level::Low);
assert_eq!(relay.level(false), Level::High);
assert!(relay.is_asserted(Level::Low));
```

- `ActiveHigh` - A high level means asserted (the direct mapping).
- `ActiveLow` - A low level means asserted (the inverted mapping).

### `Polarity::level`

Returns the physical level for a logical state.

**Arguments**

* `asserted` - whether the signal should be asserted.

**Returns**

The [`Level`] that represents that state under this polarity.

```rust
fn level(self, asserted: bool) -> Level
```

### `Polarity::is_asserted`

Returns whether a physical level means the signal is asserted.

**Arguments**

* `level` - the level read on the pin.

**Returns**

`true` if `level` asserts the signal under this polarity.

```rust
fn is_asserted(self, level: Level) -> bool
```

