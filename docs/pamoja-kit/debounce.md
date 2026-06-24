# pamoja-kit::debounce

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

Debouncing a chattering on/off signal.

## struct `Debounce`

Cleans a noisy boolean signal by requiring it to hold steady before reporting a change.

A mechanical switch, a relay contact, or a reading crossing a threshold does not flip
cleanly: for a few milliseconds it chatters between states. A [`Debounce`] reports a
change only after the new value has been seen for a set number of consecutive samples,
so a button press, a float-switch trip, or a threshold crossing reads as one clean
event. This is the standard counter debounce: N stable samples accept a change, and any
contrary sample resets the count. At a fixed sample rate, N samples is the debounce time
- sampling every 5 ms with `samples` of `4` debounces over 20 ms.

**Examples**

```
use pamoja_kit::Debounce;

// A button needs three stable samples to register.
let mut button = Debounce::new(3, false);
assert!(!button.update(true)); // first press sample
assert!(!button.update(false)); // the contact bounced back
assert!(!button.update(true)); // counting restarts
assert!(!button.update(true));
assert!(button.update(true)); // three in a row: pressed
```

### `Debounce::new`

Creates a debouncer.

**Arguments**

* `samples` - consecutive stable samples required to accept a change. `0` and `1`
  both accept a change on the first contrary sample.
* `initial` - the starting debounced state.

**Returns**

A debouncer reporting `initial` until a change is confirmed.

```rust
fn new(samples: u16, initial: bool) -> Self
```

### `Debounce::update`

Feeds a raw sample and returns the debounced state.

**Arguments**

* `raw` - the latest raw signal value.

**Returns**

The debounced state after this sample. It changes only once a contrary value has
held for the required number of samples.

```rust
fn update(&mut self, raw: bool) -> bool
```

### `Debounce::state`

Returns the current debounced state.

```rust
fn state(&self) -> bool
```

