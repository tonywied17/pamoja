# zero-edge

A single, modular SDK for IoT, robotics, and drones. One memory-safe Rust engine at the core, with idiomatic bindings for TypeScript/Node, Python, C#/.NET, Swift/Kotlin, and bare-metal Rust. Install only the capabilities you need.

Status: early planning. Nothing is published yet.

## What it is

Build and control physical devices (sensors, robots, drones, gateways) from whatever language you already use, with C-class performance and memory safety. It is designed to run well in the hard environment first: cheap hardware, low power, intermittent connectivity, and long-range radio, which makes it run well anywhere.

Pillars:

- Performant - native Rust core, async-first, small enough to run `no_std` on microcontrollers.
- Secure - memory safety by construction, TLS 1.3 / DTLS, device identity, signed OTA.
- Quality of life - one consistent API shape across every language, with both a high-level ergonomic facade and low-level access.
- Easy to adopt - opt-in scoped packages, sensible defaults, and simulators so you can build and test with zero hardware.

## Layout

The codebase is a Cargo workspace (the engine and capability crates) plus per-language binding projects published to npm, PyPI, and NuGet.

```
crates/      Rust engine + capability crates
bindings/    per-language bindings (Node, Python, .NET, Swift/Kotlin)
examples/    runnable samples per module and language
sims/        device and transport simulators for hardware-free testing
docs/        guides and generated API reference
```

## License

Released under the [MIT License](LICENSE-MIT).
