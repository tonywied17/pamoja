# source

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

The single seam between the dashboard and whatever produces its data.

A real node and the [`Mock`](crate::Mock) both implement [`StateSource`], so the
serving layer never knows which it is talking to. What you design and debug against
the mock on a laptop is exactly what ships against real sensors.

## trait `StateSource`

Produces the current [`State`] snapshot whenever the dashboard asks for one.

The serving layer calls [`snapshot`](StateSource::snapshot) to answer `GET /state`
and again on each live-update tick, so an implementation should return the latest
view of the node cheaply. It takes `&mut self` so a source may advance internal
state (a mock its clock, a real node its smoothing) as it is polled.

### `fn snapshot(&mut self) -> State`

Returns the node's current state snapshot.

**Returns**

The latest language-neutral [`State`] to render.

### `fn select(&mut self, key: &str) -> bool`

Switches a named view, for development and debugging only.

The serving layer calls this when a request carries a `?scenario=` parameter,
so a single running dev server can be flipped through every state the UI must
handle. A real node has nothing to switch, so the default ignores the request.

**Arguments**

* `key` - the requested view's identifier.

**Returns**

`true` if the source switched to `key`, `false` if it does not recognize it.

### `fn command(&mut self, command: &Command) -> Result <(), CommandError>`

Carries out an authenticated control command, changing the node's state.

The serving layer calls this only after a command has been authenticated, so an
implementation may act on it directly. A read-only source rejects every command,
which is the default.

**Arguments**

* `command` - the action to carry out.

**Returns**

`Ok(())` once the command has been applied; its effect shows in the next snapshot.

**Errors**

Returns a [`CommandError`] if the source does not support the command, the target
is unknown, or the action is not allowed.

