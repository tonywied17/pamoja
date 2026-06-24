# serve

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

An HTTP/1.1 server that serves the dashboard over a pluggable byte transport.

This is the host side of the local-first dashboard: it serves the static page, the
language-neutral `GET /state` snapshot, and a `GET /events` server-sent-event
stream for live updates, with no web framework and no async runtime. A thread per
connection keeps it simple and is ample for the handful of clients a node sees over
its own hotspot. The same code backs the [`Mock`](crate::Mock) in development and a
real node in the field, since both are just a [`StateSource`].

## Why HTTP/1.1, and the seam for more

HTTP/1.1 is the baseline on purpose: it is exactly what a browser speaks to a
device over a plain `http://` hotspot, where there is no CA-trusted certificate and
so no HTTPS (and therefore no browser HTTP/2, which is only negotiated over TLS).
It also fits the smallest tiers, where a TLS plus HTTP/2 stack would not. The
request handling is generic over any [`Read`] + [`Write`] stream, and connections
arrive through a [`Transport`], so a capable tier can later supply a TLS transport
(which negotiates HTTP/2 for free in the browser) without touching the request
logic. [`TcpTransport`] is the plain-TCP baseline; a `rustls`-backed transport is
the intended Tier A addition.

## trait `Transport`

How connections reach the server: the seam that keeps the byte transport pluggable.

The baseline is [`TcpTransport`] (plain TCP, HTTP/1.1). A capable tier can
implement this over a TLS stream so the browser negotiates HTTPS, and with it
HTTP/2, while the request handling above stays unchanged.

### `fn accept(&self) -> std::io::Result <Self::Conn>`

Blocks until the next client connects.

**Returns**

The accepted connection.

**Errors**

Returns the [`std::io::Error`] from the underlying accept call.

### `fn describe(&self) -> String`

A human-readable address for the startup log line, such as a URL.

**Returns**

The address to print when the server starts serving.

## struct `TcpTransport`

The plain-TCP, HTTP/1.1 transport: the baseline that works on any tier.

### `TcpTransport::bind`

Binds a listener on `addr`.

**Arguments**

* `addr` - the address to listen on, such as `"0.0.0.0:80"`.

**Returns**

A transport ready to accept connections.

**Errors**

Returns the [`std::io::Error`] from binding if the address is unavailable.

```rust
fn bind(addr: impl ToSocketAddrs) -> std::io::Result <Self>
```

## struct `Server`

The dashboard HTTP server, generic over whatever produces its state.

Build one with [`Server::new`], optionally set the live-update cadence with
[`Server::with_push_interval`], then block in [`Server::run`] (plain TCP) or
[`Server::run_on`] (a custom [`Transport`]).

**Examples**

```no_run
use pamoja_dashboard::{Assets, Mock, Scenario, Server};

let server = Server::new(Mock::new(Scenario::Normal), Assets::Embedded);
server.run("127.0.0.1:8080").expect("serve");
```

### `Server <S>::new`

Creates a server that renders `source` with `assets`.

Control is authenticated against a freshly generated pairing secret that nobody
holds yet, so no client can issue commands until [`with_pairing_secret`] sets the
secret the device actually shows. Read-only viewing needs no pairing.

[`with_pairing_secret`]: Server::with_pairing_secret

**Arguments**

* `source` - the state source to serve, a real node or a [`Mock`](crate::Mock).
* `assets` - where the page files come from, embedded or a directory.

**Returns**

A server pushing live updates once a second by default.

```rust
fn new(source: S, assets: Assets) -> Self
```

### `Server <S>::with_pairing_secret`

Sets the pairing secret a client must know to issue commands.

The secret is shown out of band (the device's screen, a QR code, or the dev
server's console) and never crosses the network.

**Arguments**

* `secret` - the canonical pairing secret.

**Returns**

The server, for chaining.

```rust
fn with_pairing_secret(mut self, secret: impl Into <String>) -> Self
```

### `Server <S>::with_push_interval`

Sets how often the `GET /events` stream pushes a fresh snapshot.

**Arguments**

* `interval` - the delay between pushes.

**Returns**

The server, for chaining.

```rust
fn with_push_interval(mut self, interval: Duration) -> Self
```

### `Server <S>::run`

Binds `addr` over plain TCP and serves forever.

**Arguments**

* `addr` - the address to listen on, such as `"0.0.0.0:80"`.

**Returns**

Never returns on success; it serves until the process ends.

**Errors**

Returns the [`std::io::Error`] from binding the listener.

```rust
fn run(self, addr: impl ToSocketAddrs) -> std::io::Result <()>
```

### `Server <S>::run_on`

Serves forever over a supplied [`Transport`], one thread per connection.

This is the seam for a non-default transport, such as a future TLS transport
for a capable tier.

**Arguments**

* `transport` - the source of connections.

**Returns**

Never returns on success; it serves until the process ends.

**Errors**

Returns a [`std::io::Error`] only if accepting fails unrecoverably; transient
accept errors are skipped.

```rust
fn run_on <T: Transport>(self, transport: T) -> std::io::Result <()>
```

