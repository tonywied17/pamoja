# pamoja-core

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

Core abstractions for the pamoja device SDK.

This crate defines the traits that every capability crate (for example
`pamoja-mqtt`, `pamoja-io-serial`, or `pamoja-ros2`) implements. The
core is protocol-agnostic: it models devices, sensors, actuators, transports,
durable storage, and an event bus, and leaves concrete protocol support to the
capability crates so that an application depends only on what it uses.

The primary abstractions are:

- [`Device`] - a connectable physical or virtual device.
- [`Sensor`] - a source of typed readings.
- [`Actuator`] - a sink for typed commands.
- [`Telemetry`] - a stream of telemetry frames.
- [`Transport`] - a bidirectional byte transport.
- [`Store`] - a durable store-and-forward queue.
- [`EventBus`] - a typed publish/subscribe channel.
- [`Error`] and [`Result`] - the shared error model.

**Examples**

Implementing [`Sensor`] for a temperature probe:

```
use pamoja_core::{Result, Sensor};

struct Thermometer {
    celsius: f32,
}

impl Sensor for Thermometer {
    type Reading = f32;

    async fn read(&mut self) -> Result<Self::Reading> {
        Ok(self.celsius)
    }
}

let _probe = Thermometer { celsius: 20.5 };
```

## Modules

- [bus](bus.md)
- [device](device.md)
- [error](error.md)
- [store](store.md)
- [transport](transport.md)

