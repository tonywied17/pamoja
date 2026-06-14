//! The device model: the abstractions every capability crate implements.

use crate::error::Result;

/// A physical or virtual device the SDK can talk to.
pub trait Device {
    /// A stable identifier for this device, such as a serial number or address.
    fn id(&self) -> &str;

    /// Open the device and make it ready for use.
    async fn connect(&mut self) -> Result<()>;

    /// Release the device and any underlying resources.
    async fn disconnect(&mut self) -> Result<()>;
}

/// A source of readings, such as temperature, a GPS fix, or a lidar scan.
pub trait Sensor {
    /// The reading this sensor produces.
    type Reading;

    /// Take a single reading.
    async fn read(&mut self) -> Result<Self::Reading>;
}

/// A sink that accepts commands, such as a motor setpoint or a valve state.
pub trait Actuator {
    /// The command this actuator accepts.
    type Command;

    /// Apply a command to the actuator.
    async fn apply(&mut self, command: Self::Command) -> Result<()>;
}

/// A device that emits a stream of telemetry frames.
pub trait Telemetry {
    /// The telemetry frame type.
    type Frame;

    /// Await the next telemetry frame, or `None` once the stream ends.
    async fn next_frame(&mut self) -> Result<Option<Self::Frame>>;
}
