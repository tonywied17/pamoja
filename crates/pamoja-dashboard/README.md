# pamoja-dashboard

A local-first dashboard a device serves over its own WiFi hotspot. A clinic worker, a
farmer, or a water committee opens a page on whatever cheap phone they have and sees their
fleet - sensors, alarms, battery, signal - in their own language, with no internet and no
app to install. The same dashboard a professional runs on a gateway in a city runs on a
two-dollar microcontroller in a village.

## The mental model

One rule makes all of this work: **the device ships plain data, the page does everything
else.** The device serves a small, language-neutral JSON snapshot of the fleet; the browser
draws the gauges, formats the numbers, translates the labels, and lays out right-to-left.
That keeps the device's job tiny and the page's job rich.

```
device  ──GET /state──▶  browser   (renders, formats, translates with its own Intl/CLDR)
        ◀─POST /command──           (authenticated control: open a valve, add a sensor)
```

The whole dashboard talks to one seam, the `StateSource` trait:

- `snapshot() -> State` - the fleet the page renders.
- `command(&Command)` - an authenticated control action to apply.

Anything that implements `StateSource` can drive the dashboard. Two do:

- **`Mock`** - a hardware-free demo fleet for development and the hosted showcase. Behind the
  `mock` feature (on by default).
- **`Fleet`** - the real source a project fills. This is what you use on a real device.

## Using it with your project

A real project owns its own sensing (it ticks its profiles/sensors on their power schedule)
and **pushes** results into a `Fleet`; the dashboard reads them and hands control commands
back for the project to apply. This push model is why the dashboard works with any project
and stays synchronous and dependency-light.

```rust
use pamoja_dashboard::{Assets, Fleet, LinkKind, Reading, Sensor, Server, Status};

// 1. Declare the fleet's shape. The reading here is only the starting value shown until the
//    first real sample arrives - it is not a fixed value; live values are fed in step 2.
let fleet = Fleet::builder()
    .org("farm", "Pamoja farm")
    .group("farm", "field", "Field node", LinkKind::Lora)
    .sensor("field", Sensor::new("soil", Reading::new("soil_moisture", 60.0, "percent").with_band(40.0, 80.0)))
    .build();

// 2. From your own sampling loop, feed each real reading in. The Fleet keeps the rolling
//    history (the sparkline) for you, and queues any control commands for you to apply.
let worker = fleet.clone();
loop {
    let value = read_soil_sensor();  // your driver, or a pamoja-sensors decoder
    worker.report_reading(
        "field",
        "soil",
        Reading::new("soil_moisture", value, "percent")
            .with_band(40.0, 80.0)
            .with_status(if value < 40.0 { Status::Warn } else { Status::Ok }),
    );
    for command in worker.take_commands() { /* drive real hardware, then report the result */ }
    // wait for the sensor's duty cycle
}

// 3. Serve it (from another thread or task; `run` blocks).
Server::new(fleet, Assets::Embedded).with_pairing_secret(secret).run("0.0.0.0:80").unwrap();
```

The builder *declares* what exists (and a starting reading); `report_reading` *feeds* live
values and grows the history automatically. That split is why the dashboard works with any
project - it never reaches into your sensors; you push what you have.

A complete, runnable version is in [`examples/gateway.rs`](examples/gateway.rs) (driven by a
real `pamoja-profile` controller, with discovery and persistence). Run it:

```
cargo run -p pamoja-dashboard --example gateway
```

It also shows the gaps a real deployment fills: an added sensor carries an optional hardware
**binding** (`i2c:0x76`, `gpio:4`, `lora:ab12`) for the gateway to bind a driver;
`Fleet::add_sensor`/`add_group` surface a node the moment it is discovered; and
`Fleet::from_state` + `State::from_json` restore a fleet across restarts.

## Authenticated control

Reading is anonymous; moving an actuator or changing the fleet is not, because the hotspot is
open. The device shows a pairing secret out of band (its screen, a QR, or the dev console);
the browser mixes it with a server nonce into a session key (HKDF) and signs every command
with a counter and an HMAC, so an on-network attacker can neither forge a command nor replay
a captured one, and the secret never crosses the network. The keyed-hash primitives are
reused from `pamoja-session`; the browser ships a tiny pure-JS SHA-256/HMAC because WebCrypto
is unavailable over plain `http://`.

## Localization

Translations live once, as one JSON file per locale under
[`web/app/i18n/`](web/app/i18n/) - the single source the browser fetches and renders with its
own CLDR-backed `Intl` (plurals, numbering systems, right-to-left). There is no generation
step and nothing to keep in sync by hand. `cargo xtask dashboard i18n` validates the bundles
(key, placeholder, and metadata parity; gzipped footprint).

## Performance

The device serves its assets gzip-encoded, and `cargo xtask dashboard footprint` enforces a
gzipped page-load budget so the bundle stays small over a weak link (currently well under
150 KB including one locale). First paint needs no round trip after the initial load.

## Build modes: real vs demo

The crate is feature-gated so a real firmware build ships no demo:

| Build | Features | What you get |
| --- | --- | --- |
| Real device | `--no-default-features --features serve` | `Fleet`, `Server`, control - no mock fleet, no scenario switcher |
| Development / showcase | default (`serve`, `mock`) | `Mock` + the dev server + the static snapshot generator |

Snapshots from the mock carry a `demo` flag; the page shows demo-only affordances (the
scenario switcher) only when it is set, so a real device never exposes them.

## Web app

The page is a multi-file [zQuery](../../) app under [`web/`](web/): `index.html`,
`global.css`, the vendored `zquery.min.js`, and `app/` (entry, store, router, the live feed,
i18n, the pairing/crypto helpers, the visualizations, and the components). In development
`Assets::Dir` serves it from disk with hot reload; in production `Assets::Embedded` bakes it
into the binary with `include_bytes!`.

## Commands

- `cargo xtask dashboard dev [scenario]` - run the mock-backed dev server (hot reload).
- `cargo run -p pamoja-dashboard --example gateway` - run the real-`Fleet` reference gateway.
- `cargo xtask dashboard i18n --check` - validate the locale bundles.
- `cargo xtask dashboard footprint` - check the gzipped page-load budget.
- `cargo xtask docs` - regenerate the workspace API reference under the repo's `docs/`.

## API reference

The canonical per-item API is on docs.rs once the crate is published. A Markdown mirror of the
whole workspace, generated from the rustdoc by `cargo xtask docs`, lives in the repo under
[`docs/pamoja-dashboard/`](../../docs/pamoja-dashboard/README.md).
