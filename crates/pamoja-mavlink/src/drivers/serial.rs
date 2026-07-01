//! A serial-port [`ByteLink`], for a companion computer wired to a flight controller.
//!
//! A Pixhawk-class autopilot speaks MAVLink over a UART, so a companion computer (a Raspberry Pi
//! or similar) reaches it through a serial port at a fixed baud rate. [`SerialLink`] opens such a
//! port on top of `tokio-serial`. It is available behind the `serial` feature, which pulls in the
//! serial-port backend.

use std::io;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_serial::{SerialPortBuilderExt, SerialStream};

use super::link_fault;
use crate::error::Result;
use crate::link::ByteLink;

/// A serial port presented as a [`ByteLink`].
pub struct SerialLink {
    port: SerialStream,
}

impl SerialLink {
    /// Opens a serial port at a baud rate.
    ///
    /// # Arguments
    ///
    /// * `path` - the port to open, such as `"/dev/ttyACM0"` or `"COM3"`.
    /// * `baud` - the baud rate, commonly `57600` or `115200` for a flight controller.
    ///
    /// # Returns
    ///
    /// The opened link.
    ///
    /// # Errors
    ///
    /// Returns an [`io::Error`] if the port cannot be opened.
    pub fn open(path: &str, baud: u32) -> io::Result<Self> {
        let port = tokio_serial::new(path, baud)
            .open_native_async()
            .map_err(io::Error::from)?;
        Ok(SerialLink { port })
    }
}

impl ByteLink for SerialLink {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.port.read(buf).await.map_err(link_fault)
    }

    async fn write_all(&mut self, data: &[u8]) -> Result<()> {
        self.port.write_all(data).await.map_err(link_fault)
    }
}
