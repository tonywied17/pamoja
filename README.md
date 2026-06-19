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

The engine and capability crates and the language bindings below are available
today.

### Engine and capability crates

| Crate | Area | What it does |
| --- | --- | --- |
| `pamoja-core` | core | The device model: `Transport`, `Device`, `Sensor`, `Actuator`, `Store`, event-bus, and error traits. |
| `pamoja-codec` | serialize | CBOR, JSON, and raw codecs behind one trait, plus delta+varint batch packing and an `f32` quantizer for metered links. |
| `pamoja-mqtt` | messaging | An MQTT client implementing the core `Transport` trait, tested against an embedded broker. |
| `pamoja-coap` | messaging | A CoAP client over UDP with confirmable and non-confirmable delivery and RFC 7641 observe. |
| `pamoja-ladder` | resilience | A cost-aware transport ladder: cheapest reachable rung first, buffering to a `Store` when every link is down. |
| `pamoja-sync` | resilience | Offline-first store-and-forward queues: in-memory, plus a crash-safe on-disk queue that survives power loss. |
| `pamoja-bus` | core | An in-memory typed publish/subscribe event bus implementing the core `EventBus` trait. |
| `pamoja-loopback` | testing | An in-process `Transport` with topic matching and a fault injector, exercising the full path with no broker. |
| `pamoja-sim` | testing | Hardware-free simulators: noisy and replay sensors, a recording actuator, and a degraded-link transport. |
| `pamoja-power` | energy | Duty cycling plus an energy-aware governor that stretches work as the battery drains and eases off while charging. |
| `pamoja-security` | trust | ed25519 device identity: sign a device's telemetry and verify it, so a gateway can prove a reading is authentic. |
| `pamoja-audit` | trust | A `no_std` tamper-evident, SHA-256 hash-chained log; altering, reordering, or dropping any record breaks verification. |
| `pamoja-telemetry` | observe | Allocation-free observability that ships only what is worth the bytes as link cost rises, while counting everything. |
| `pamoja-lora` | radio | The exact LoRa time-on-air of a payload and the duty-cycle off-time it forces, so a node stays in regulation and budget. |
| `pamoja-lorawan` | radio | LoRaWAN 1.0.x MAC framing with AES-CMAC and AES encryption and OTAA join, against the FIPS-197 and RFC 4493 vectors. |
| `pamoja-mesh` | mesh | Addressed, hop-limited, CRC-checked frames plus duplicate suppression that floods a packet across the mesh exactly once. |
| `pamoja-routing` | mesh | Reverse-path routing that learns the cheapest route from overheard traffic, saving the airtime flooding wastes. |
| `pamoja-modbus` | field I/O | Modbus RTU framing (CRC-16/Modbus) with request builders and reply decoders for RS485 field sensors. |
| `pamoja-can` | field I/O | CAN 2.0 and CAN-FD frames (11- and 29-bit IDs) plus J1939 decode and compose for trucks, tractors, and gensets. |
| `pamoja-serial` | field I/O | SLIP (RFC 1055) and COBS byte-stuffing with streaming frame decoders, so a raw UART byte stream carries discrete packets to motor controllers, GPS, and LiDAR. |
| `pamoja-gpio` | field I/O | On-board bus logic: I2C 7- and 10-bit address frames (NXP UM10204) with reserved-range checks, the four SPI clock modes, and active-high/active-low GPIO pins. |
| `pamoja-kit` | ergonomics | Plain-language helpers that name the goal over the math: smooth, calibrate, keep, warn, geofence, rolling-window stats, and real-world unit conversions. |
| `pamoja-profile` | ergonomics | Named, ready-to-run device profiles from plain data or a JSON manifest; assembled and testable with no hardware. |
| `pamoja-ffi` | bindings | The curated C ABI over the core and MQTT, with a `cbindgen`-generated, drift-checked `pamoja.h`. |

### Language bindings

| Package | Language | What it is |
| --- | --- | --- |
| `@pamoja/core` | TypeScript / Node | A generated contract plus a hand-written TypeScript facade (napi-rs). |
| `pamoja-core` | Python | A generated, type-stubbed contract plus a hand-written async facade (PyO3 + maturin). |
| `Pamoja.Core` | C# / .NET | A P/Invoke interop layer plus an async facade with `IAsyncEnumerable` streams and `IAsyncDisposable` lifecycle. |

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

Messaging and radio. MQTT and CoAP work today, behind a cost-aware transport ladder that tries the cheapest link first and buffers when there is none. LoRa and LoRaWAN long-range radio, and a CRC-checked mesh frame with reverse-path routing, now ship as further rungs. Next: the cheap-radio drivers they ride on (ESP-NOW, nRF24), a Meshtastic bridge for off-grid networks, and cellular and satellite uplinks for the most remote telemetry.

Hardware and sensors. Serial (SLIP/COBS), CAN with J1939, and RS485/Modbus ship today for long field cabling, with GPIO/I2C/SPI next. A catalog of drivers for cheap, common, salvageable parts, plus device profiles you instantiate by name (an irrigation node, a well-level monitor) instead of wiring pins.

Resilience and power. Offline-first store-and-forward and energy-aware duty cycling for solar and battery work today; next are local-first dashboards a device serves over its own hotspot, and data-mule sync for places with no link at all.

Robotics and drones. A ROS2 and Zenoh bridge, then MAVLink for drones, modeled as ordinary pamoja devices.

Security. Memory safety by construction today, with ed25519 device identity and a tamper-evident, hash-chained audit log already shipping. Next: TLS 1.3 and DTLS, X.509 device identity, and signed OTA updates with verified rollback.

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
