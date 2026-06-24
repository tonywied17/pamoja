# auth

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

Authenticating control over an open hotspot.

Reading the dashboard is anonymous; moving an actuator or changing the fleet is not.
Because the serving hotspot is unencrypted, control cannot rely on a bearer token a
sniffer could capture and replay. Instead the device holds a pairing secret shown out
of band (its own screen, a QR code, or the dev server's console). A client that knows
the secret derives a per-session key from it and a server nonce, and authenticates
every command with a counter and an HMAC, so an on-network attacker can neither forge
a command nor replay a captured one. The secret itself never crosses the network.

The keyed-hash primitives are reused from [`pamoja_session`] so this shares one
audited, vector-pinned crypto path. Sessions live in memory; a server restart simply
requires re-pairing.

## enum `AuthError`

Why a control request was refused. The [`code`](AuthError::code) is a stable,
language-neutral string the page localizes.

- `UnknownSession` - No session with that id exists; the client must pair first.
- `NotPaired` - The session exists but has not completed pairing.
- `Expired` - The session has expired; the client must pair again.
- `Replayed` - The counter is not greater than the last accepted one (a replay or reorder).
- `BadMac` - The supplied MAC does not match; the client does not hold the session key.

### `AuthError::code`

Returns the stable error code for this failure.

**Returns**

A dotted, language-neutral code such as `"auth.bad_mac"`.

```rust
fn code(self) -> &'static str
```

## struct `Challenge`

A pairing challenge handed to a client in the clear.

Fields:

- `session_id: String` - The opaque session identifier the client echoes on confirm and every command.
- `nonce: String` - The per-session salt the client mixes with the pairing secret to derive the key.

## struct `Auth`

Gatekeeper for control actions: it issues pairing challenges and verifies commands.

### `Auth::new`

Creates an authenticator for a pairing secret.

**Arguments**

* `secret` - the canonical pairing secret string (the client normalizes a typed
  code to the same value).

**Returns**

An authenticator with no sessions yet.

```rust
fn new(secret: impl Into <String>) -> Self
```

### `Auth::generate_secret`

Generates a fresh high-entropy pairing secret as lowercase hex.

**Returns**

A 128-bit secret rendered as 32 hex characters.

```rust
fn generate_secret() -> String
```

### `Auth::challenge`

Starts a pairing exchange, returning a challenge and recording an unconfirmed
session.

**Returns**

The [`Challenge`] to send to the client.

```rust
fn challenge(&self) -> Challenge
```

### `Auth::confirm`

Confirms a pairing by checking the client proved it derived the session key.

**Arguments**

* `session_id` - the challenge's session id.
* `mac_hex` - `HMAC(key, "confirm\n" + session_id)` as lowercase hex.

**Returns**

`Ok(())` if the proof is valid and the session is now paired.

**Errors**

[`AuthError::UnknownSession`], [`AuthError::Expired`], or [`AuthError::BadMac`].

```rust
fn confirm(&self, session_id: &str, mac_hex: &str) -> Result <(), AuthError>
```

### `Auth::verify_command`

Verifies an authenticated command and advances the session's replay counter.

The MAC covers the counter and the exact command string, so the server checks the
same bytes the client signed without re-serializing.

**Arguments**

* `session_id` - the paired session's id.
* `counter` - the strictly increasing per-session command counter.
* `command` - the exact command payload string the client signed.
* `mac_hex` - `HMAC(key, counter + "\n" + command)` as lowercase hex.

**Returns**

`Ok(())` if the command is authentic and fresh; the counter is then recorded.

**Errors**

[`AuthError::UnknownSession`], [`AuthError::NotPaired`], [`AuthError::Expired`],
[`AuthError::Replayed`], or [`AuthError::BadMac`].

```rust
fn verify_command(&self, session_id: &str, counter: u64, command: &str, mac_hex: &str,) -> Result <(), AuthError>
```

