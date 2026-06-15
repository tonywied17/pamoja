//! Device-model traits implemented by capability crates.
//!
//! These traits describe the roles a piece of hardware can play in an
//! application: a connectable [`Device`], a [`Sensor`] that produces readings, an
//! [`Actuator`] that accepts commands, and a [`Telemetry`] source that streams
//! frames. A single type may implement more than one of them.

use crate::error::Result;

/// A connectable physical or virtual device.
///
/// Implementors manage the lifecycle of an underlying resource such as a serial
/// port, a network socket, or a vehicle link.
pub trait Device {
    /// Returns the stable identifier for this device.
    ///
    /// The identifier is expected to remain constant for the lifetime of the
    /// device, for example a serial number, MAC address, or vehicle URI.
    ///
    /// # Returns
    ///
    /// A string slice borrowing the device's identifier.
    fn id(&self) -> &str;

    /// Opens the device and prepares it for use.
    ///
    /// # Returns
    ///
    /// `Ok(())` once the device is connected and ready.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`](crate::Error::Io) if the underlying resource cannot
    /// be opened, or [`Error::Transport`](crate::Error::Transport) if a link to
    /// the device cannot be established.
    async fn connect(&mut self) -> Result<()>;

    /// Releases the device and any resources it holds.
    ///
    /// # Returns
    ///
    /// `Ok(())` once the device has been disconnected and its resources freed.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`](crate::Error::Io) if the underlying resource cannot
    /// be released cleanly.
    async fn disconnect(&mut self) -> Result<()>;
}

/// A source of typed readings, such as a thermometer, GPS receiver, or lidar.
pub trait Sensor {
    /// The value produced by a single read, for example a temperature or a fix.
    type Reading;

    /// Takes a single reading from the sensor.
    ///
    /// # Returns
    ///
    /// The next [`Reading`](Self::Reading) sampled from the sensor.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`](crate::Error::Io) if the sensor cannot be read, or
    /// [`Error::Closed`](crate::Error::Closed) if the sensor has been
    /// disconnected.
    async fn read(&mut self) -> Result<Self::Reading>;
}

/// A sink that accepts typed commands, such as a motor or a valve.
pub trait Actuator {
    /// The command accepted by a single application, for example a setpoint.
    type Command;

    /// Applies a command to the actuator.
    ///
    /// # Arguments
    ///
    /// * `command` - the command to apply, consumed by the call.
    ///
    /// # Returns
    ///
    /// `Ok(())` once the command has been accepted by the actuator.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`](crate::Error::Io) if the command cannot be
    /// delivered, or [`Error::Closed`](crate::Error::Closed) if the actuator has
    /// been disconnected.
    async fn apply(&mut self, command: Self::Command) -> Result<()>;
}

/// A device that emits a continuous stream of telemetry frames.
pub trait Telemetry {
    /// A single telemetry frame, for example a status or position report.
    type Frame;

    /// Awaits the next telemetry frame.
    ///
    /// # Returns
    ///
    /// `Some(frame)` when a frame is available, or `None` once the telemetry
    /// stream has ended.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Transport`](crate::Error::Transport) if the telemetry
    /// link fails while waiting.
    async fn next_frame(&mut self) -> Result<Option<Self::Frame>>;
}
