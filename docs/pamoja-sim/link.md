# pamoja-sim::link

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

A transport decorator that simulates a degraded radio link.

## struct `DegradedLink`

A [`Transport`] decorator that simulates an ongoing degraded link.

Where `Faulty` in `pamoja-loopback` fails a fixed number of upcoming sends, a
`DegradedLink` models a link that stays bad, so offline-first behavior can be
proven against a realistic pattern rather than a one-shot outage. It can drop a
configurable fraction of sends (a lossy radio) and cycle between reachable and
unreachable windows (a link that comes and goes). Both are deterministic - driven
by a send counter, not a clock or randomness - so a store-and-forward drain over
the link behaves the same way every run.

Connect and subscribe pass straight through; only [`send`](Transport::send) is
degraded, since that is the path store-and-forward depends on. A degraded send
returns [`Error::Transport`], which [`drain_to`](https://docs.rs/pamoja-sync)
leaves buffered, in order, to retry later.

**Examples**

```
use pamoja_core::Transport;
use pamoja_loopback::{LoopbackBroker, LoopbackTransport};
use pamoja_sim::DegradedLink;

let broker = LoopbackBroker::new();
// A link that drops every second packet.
let mut link = DegradedLink::new(LoopbackTransport::new(broker)).drop_every(2);
link.connect().await?;

link.send("t", b"1").await?; // first send: delivered
assert!(link.send("t", b"2").await.is_err()); // second send: dropped
link.send("t", b"3").await?; // third send: delivered
```

### `DegradedLink <T>::new`

Wraps `inner` as a perfect link, until loss or intermittency is added.

**Arguments**

* `inner` - the transport to decorate.

**Returns**

A decorator that passes every send through until configured otherwise.

```rust
fn new(inner: T) -> Self
```

### `DegradedLink <T>::drop_every`

Drops one in every `n` sends, simulating a lossy link.

**Arguments**

* `n` - drop every `n`th send; `0` disables loss.

**Returns**

The updated link, for chaining.

```rust
fn drop_every(mut self, n: u32) -> Self
```

### `DegradedLink <T>::intermittent`

Cycles between `up` reachable sends and `down` unreachable sends.

Sends rejected during a down window return [`Error::Transport`], the same as a
real link that is temporarily out of range.

**Arguments**

* `up` - the number of sends that succeed at the start of each cycle.
* `down` - the number of sends that fail before the cycle repeats.

**Returns**

The updated link, for chaining.

```rust
fn intermittent(mut self, up: u32, down: u32) -> Self
```

### `DegradedLink <T>::into_inner`

Unwraps the decorator, returning the inner transport.

**Returns**

The wrapped transport.

```rust
fn into_inner(self) -> T
```

