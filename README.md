<div align="center">

<img src="assets/pamoja-logo.svg" alt="pamoja" width="620">

**One memory-safe Rust core. Every language. For the devices that change lives.**

<picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/tonywied17/pamoja/main/.github/badges/tag-iot-dark.svg"><img height="26" alt="IoT" src="https://raw.githubusercontent.com/tonywied17/pamoja/main/.github/badges/tag-iot-light.svg"></picture>
&nbsp;<picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/tonywied17/pamoja/main/.github/badges/tag-robotics-drones-dark.svg"><img height="26" alt="robotics &amp; drones" src="https://raw.githubusercontent.com/tonywied17/pamoja/main/.github/badges/tag-robotics-drones-light.svg"></picture>
&nbsp;<picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/tonywied17/pamoja/main/.github/badges/tag-edge-first-dark.svg"><img height="26" alt="edge-first" src="https://raw.githubusercontent.com/tonywied17/pamoja/main/.github/badges/tag-edge-first-light.svg"></picture>
&nbsp;<picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/tonywied17/pamoja/main/.github/badges/tag-offline-first-dark.svg"><img height="26" alt="offline-first" src="https://raw.githubusercontent.com/tonywied17/pamoja/main/.github/badges/tag-offline-first-light.svg"></picture>
&nbsp;<picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/tonywied17/pamoja/main/.github/badges/tag-rust-core-dark.svg"><img height="26" alt="Rust core" src="https://raw.githubusercontent.com/tonywied17/pamoja/main/.github/badges/tag-rust-core-light.svg"></picture>

<a href="https://crates.io/users/tonywied17"><img height="26" alt="crates.io" src="https://raw.githubusercontent.com/tonywied17/pamoja/main/.github/badges/btn-crates.svg"></a>
&nbsp;<a href="https://www.npmjs.com/org/pamoja"><img height="26" alt="npm" src="https://raw.githubusercontent.com/tonywied17/pamoja/main/.github/badges/btn-npm.svg"></a>
&nbsp;<a href="https://pypi.org/user/tonywied17/"><img height="26" alt="PyPI" src="https://raw.githubusercontent.com/tonywied17/pamoja/main/.github/badges/btn-pypi.svg"></a>
&nbsp;<a href="https://www.nuget.org/profiles/tonywied17"><img height="26" alt="NuGet" src="https://raw.githubusercontent.com/tonywied17/pamoja/main/.github/badges/btn-nuget.svg"></a>
&nbsp;<a href="LICENSE-MIT"><img height="26" alt="MIT license" src="https://raw.githubusercontent.com/tonywied17/pamoja/main/.github/badges/btn-license.svg"></a>

</div>

## What is pamoja

pamoja is a single, modular SDK for IoT, robotics, and drones: one memory-safe Rust engine at the core, with idiomatic bindings for the languages a device developer actually uses. You install only the capabilities you need, and the same concepts work the same way in every language.

Control and communicate with physical things - sensors, robots, drones, gateways - from TypeScript, Python, C#, Lua, or Rust itself, with C-class performance and memory safety, without hand-rolling FFI.

## Why it exists

The places where connected devices can do the most good - smallholder farms, off-grid villages, rural clinics, disaster zones - are exactly the places with the least money, the worst connectivity, and the cheapest hardware. Most IoT and robotics stacks quietly assume the opposite of all of that.

pamoja is built for the hard environment first. If it runs well on a two-dollar microcontroller on a solar panel with an intermittent radio link, it runs well anywhere. That single constraint makes the library better for everyone.

What that means in practice:

- Cheap and salvageable hardware, down to microcontrollers with a few hundred KB of RAM.
- Offline-first: local buffering and store-and-forward, so a device disconnected for days loses nothing.
- Low bandwidth and long range: compact codecs and radio (LoRa, mesh) treated as first-class.
- Low power: async duty-cycling and energy-aware scheduling for battery and solar.
- Free and unencumbered, so cost is never a barrier to use.
- Reachable: many languages, plus a plain-language helper layer so you do not need to be an engineer to build something that works.

## The pillars

- Performant - native Rust, async-first, small enough to run `no_std` on microcontrollers.
- Secure - memory safety by construction, TLS 1.3 / DTLS, device identity, signed OTA.
- Quality of life - one consistent API in every language, with a high-level ergonomic facade plus a low-level escape hatch.
- Easy to adopt - opt-in scoped packages, strong defaults, and simulators so you can build and test with zero hardware.

## Status

In active development, the following crates and bindings are available:

- `pamoja-core` - the device model and transport, store, event-bus, and error traits.
- `pamoja-codec` - the pluggable serialization trait with serde-based CBOR, JSON, and raw-bytes codecs behind feature flags, plus delta-and-varint batch encoding and an `f32` quantizer that pack a batch of readings into a fraction of the bytes for metered links.
- `pamoja-mqtt` - an MQTT client implementing the core `Transport` trait, tested against an embedded broker.
- `pamoja-coap` - a CoAP client implementing the core `Transport` trait over UDP, with confirmable and non-confirmable delivery and RFC 7641 observe, tested against an in-process server.
- `pamoja-sync` - offline-first store-and-forward queues implementing the core `Store` trait: an in-memory queue and a crash-safe on-disk queue that survives power loss.
- `pamoja-loopback` - an in-process `Transport` with MQTT-style topic matching, plus a fault-injecting decorator, so examples and tests exercise the full publish/subscribe path and degraded-link behavior with no broker and no hardware.
- `pamoja-sim` - hardware-free simulators behind the core traits: a fake sensor with seedable noise and drift, a replay sensor for scripted readings, a recording actuator, and a degraded-link `Transport` decorator that drops and intermittently blocks sends, so the full stack and its offline-first behavior run and are tested with no hardware.
- `pamoja-ladder` - a cost-aware transport ladder: rank transports cheapest-first, send over the first reachable rung, and buffer to a `Store` when every link is down, draining the backlog in order once one returns.
- `pamoja-bus` - an in-memory typed publish/subscribe event bus implementing the core `EventBus` trait, broadcasting each event to every subscriber.
- `pamoja-kit` - a `no_std` helper layer that names the math for the goal, not the technique: smooth a noisy reading, turn a raw reading into real units, keep a temperature, warn before a tank runs dry, catch a reading that changes dangerously fast, and geofence a tracked point (great-circle distance), each documenting the real algorithm one layer down. The geo helpers sit behind a default feature that adds `libm`; the rest stay allocation-free and dependency-free.
- `pamoja-power` - a `no_std` power-scheduling layer: duty cycling with a duty-fraction power proxy, and an energy-aware governor that stretches the work interval as the battery drains and eases off when the panel is charging.
- `pamoja-security` - a `no_std` device-identity layer: ed25519 keys that sign a device's telemetry and verify it, so a gateway can prove a reading is authentic and unaltered. The data-integrity and audit-trail foundation for the cold-chain and metering deployments, ahead of transport TLS/DTLS and signed OTA.
- `pamoja-audit` - a `no_std` tamper-evident log: signed, SHA-256 hash-chained entries where each commits to the one before, so altering, reordering, inserting, or dropping any record breaks verification. The append-only audit trail a vaccine fridge's temperature history needs to stand as evidence.
- `pamoja-telemetry` - a `no_std`, allocation-free observability layer: structured leveled events and a reporter that ships only what is worth the bytes as the link cost rises, while counting everything so a compact periodic snapshot stays complete. Device-side observability that degrades gracefully on metered links.
- `pamoja-lora` - a `no_std`, allocation-free LoRa link model: the exact integer time-on-air of a payload and the duty-cycle off-time it forces, so a long-range node stays within regulations and budgets its power. The link-budget half of LoRa support, ahead of the radio driver.
- `pamoja-profile` - named, ready-to-run device profiles: pick a preset like the vaccine-fridge monitor or load a profile from a shareable JSON manifest, hand it a sensor, actuator, transport, and codec, and the assembled node reads, decides with the `pamoja-kit` controllers, drives the output, and publishes on the profile's `pamoja-power` schedule. The profile is plain data and the control policy is pure and hardware-free, so a whole profile is shareable and testable with no devices.
- `pamoja-ffi` - the curated C ABI over the core and MQTT, with a `cbindgen`-generated, drift-checked `pamoja.h`. This is the single auditable unsafe boundary and the seam C, C++, and .NET consume.
- `@pamoja/core` - the Node binding, shipped in two tiers: a generated contract and a hand-written TypeScript facade (the `MqttClient` today, until the capability-scoped packages land).
- `pamoja-core` (Python) - the Python binding, same two tiers: a generated, type-stubbed contract and a hand-written async facade, built with PyO3 and maturin.
- `Pamoja.Core` (.NET) - the C#/.NET binding over the C ABI, same two tiers: a P/Invoke interop layer and a hand-written async facade with `IAsyncEnumerable` message streams and `IAsyncDisposable` lifecycle.

CI runs formatting, clippy, and tests for the workspace, builds the Node, Python, and .NET bindings, and fails if any generated surface (the binding contracts and the C header) drifts from the Rust source. Release workflows publish to crates.io, npm, PyPI, and NuGet on a version tag. Everything past this is on the roadmap below.

## A quick look

TypeScript, through the ergonomic facade:

```ts
import { MqttClient } from '@pamoja/core'

const client = new MqttClient({ clientId: 'sensor-1', host: 'localhost', port: 1883 })
await client.connect()
await client.subscribe('sensors/+/temperature')
await client.publish('sensors/1/temperature', '21.5')

for await (const message of client) {
  console.log(message.topic, message.payload.toString())
}
```

The same shape in Python, through its async facade:

```python
import asyncio
from pamoja import MqttClient

async def main():
    async with MqttClient(client_id="sensor-1", host="localhost", port=1883) as client:
        await client.subscribe("sensors/+/temperature")
        await client.publish("sensors/1/temperature", "21.5")
        async for message in client:
            print(message.topic, message.payload.decode())

asyncio.run(main())
```

The same shape in C#, through its async facade:

```csharp
using Pamoja.Core;

await using var client = new MqttClient(new MqttClientOptions
{
    ClientId = "sensor-1",
    Host = "localhost",
    Port = 1883,
});
await client.ConnectAsync();
await client.SubscribeAsync("sensors/+/temperature");
await client.PublishAsync("sensors/1/temperature", "21.5");

await foreach (var message in client)
{
    Console.WriteLine($"{message.Topic}: {message.Payload.Length} bytes");
}
```

The same thing in Rust:

```rust
use pamoja_core::Transport;
use pamoja_mqtt::{MqttConfig, MqttTransport};

let mut transport = MqttTransport::new(MqttConfig::new("sensor-1", "localhost", 1883));
transport.connect().await?;
transport.subscribe("sensors/+/temperature").await?;
transport.send("sensors/1/temperature", b"21.5").await?;
```

## Architecture

Every domain capability is a separate crate behind a trait defined in the core. The core knows about `Transport`, `Device`, `Sensor`, `Actuator`, `Store`, and the event bus; it knows nothing about MQTT or CAN specifically. Concrete crates implement those traits and are pulled in only when needed, so nobody pays for what they do not use, and on a microcontroller you compile in two crates and nothing else.

This separation is literal in Rust: `pamoja-core` defines the traits, and each transport (`pamoja-mqtt`, `pamoja-coap`) is its own crate. That is why the Rust example above pulls `MqttTransport` from `pamoja-mqtt`, not from the core. The language bindings are heading to the same shape, with capability-scoped packages (`@pamoja/mqtt`, `pamoja-mqtt`, `Pamoja.Mqtt`) sitting next to the core package. Today, while the polyglot release pipeline is being proven end to end with a single capability, that first transport ships inside each language's `core` package, which is why the TypeScript, Python, and C# examples above import `MqttClient` from it. Splitting the bindings into scoped packages is on the roadmap.

```
        bindings (two tiers: generated contract + hand-written facade)
   npm @pamoja/*   PyPI pamoja-*   NuGet Pamoja.*   Lua / WASM / Kotlin / Swift
        |                |               |                    |
        +----------------+---------------+--------------------+
                                  |
                         +--------+--------+   async runtime, device model,
                         |   pamoja-core   |   event bus, error model, codecs
                         +--------+--------+
                                  |  trait-based abstraction layer
   messaging   hardware I/O   robotics    drones    security   resilience   power
   mqtt/coap   serial/can/    ros2/       mavlink   tls/       store-and-   duty-
   lora/mesh   gpio/rs485     zenoh                 identity   forward      cycling
```

## Roadmap

Messaging and radio. MQTT and CoAP work today, behind a cost-aware transport ladder that tries the cheapest link first and buffers when there is none. Next: LoRa and LoRaWAN, cheap local mesh (ESP-NOW, nRF24), a Meshtastic bridge for off-grid networks, and cellular uplink, each adding another rung to that ladder.

Hardware and sensors. Serial, CAN, GPIO/I2C/SPI, and RS485/Modbus for long field cabling. A catalog of drivers for cheap, common, salvageable parts, plus device profiles you instantiate by name (a vaccine-fridge monitor, an irrigation node, a well-level sensor) instead of wiring pins.

Resilience and power. Offline-first store-and-forward and energy-aware duty cycling for solar and battery work today; next are local-first dashboards a device serves over its own hotspot, and data-mule sync for places with no link at all.

Robotics and drones. A ROS2 and Zenoh bridge, then MAVLink for drones, modeled as ordinary pamoja devices.

Security. TLS 1.3 and DTLS, X.509 device identity, and signed OTA updates with verified rollback.

Reach. Bindings beyond Node: Python, C#/.NET, Lua, WebAssembly, Kotlin, Swift, and Go. The plain-language helper layer (`pamoja-kit`) has its first slice today - keep a temperature, smooth a noisy reading, warn before a tank runs dry - each naming the goal over the math with the real algorithm one layer down; more helpers (calibration curves, geo, control) follow. And an offline-first community cookbook so the SDK reaches the people it is built for.

## Languages

| Language | Package | Status |
| --- | --- | --- |
| Rust | `pamoja-core`, `pamoja-mqtt`, ... | available |
| TypeScript / Node | `@pamoja/core` | in progress |
| Python | `pamoja-core` | in progress |
| C# / .NET | `Pamoja.Core` | in progress |
| Lua | embeddable | planned |
| WebAssembly | browser / npm | planned |
| Kotlin, Swift, Go | platform-native | planned |

## Repository layout

```
crates/      Rust engine and capability crates (including pamoja-ffi, the C ABI)
bindings/    per-language bindings (Node, Python, .NET today; more to come)
assets/      brand and logo
```

Planned as the project grows: `examples/` (runnable samples per module and language), `sims/` (device and transport simulators for hardware-free testing), and `docs/` (guides and generated API reference).

## Building

```sh
cargo build --workspace      # build the engine and capability crates
cargo test --workspace       # run tests, including doctests and the MQTT round-trip

cd bindings/node
npm install && npm run build  # build the native addon and the TypeScript facade
npm test                      # smoke-test the binding

cd ../python
python -m venv .venv && . .venv/bin/activate
pip install maturin pytest && maturin develop  # build the extension and install the facade
pytest                                          # smoke-test the binding

cd ../..
cargo build -p pamoja-ffi --release                       # build the native C ABI and refresh pamoja.h
dotnet build bindings/dotnet/Pamoja.Core.sln -c Release    # build the .NET interop and facade
dotnet run --project bindings/dotnet/tests/Pamoja.Core.Smoke -c Release  # smoke-test the binding
```

The local toolchain needs no extra components; formatting and clippy run in CI.

## License

Released under the [MIT License](LICENSE-MIT). Free to use, with no legal or financial barrier, because cost should never be the reason a good idea does not get built.
