//! Low-level serial port handle wrapper.
//!
//! Wraps the `serialport` crate's `SerialPort` trait object,
//! providing a simpler interface for open/close/read/write operations.

use std::{path::Path, time::Duration};

use serialport::SerialPort;

use super::SerialError;

/// Wrapper around a serial port connection.
///
/// Holds an optional boxed `SerialPort` trait object. When `None`, the port is closed.
/// Can be cloned via `try_clone()` to create separate reader/writer handles.
#[derive(Default)]
pub struct PortHandle {
    /// The underlying serial port, or `None` if closed
    handle: Option<Box<dyn SerialPort>>,
}

impl PortHandle {
    /// Creates a new closed port handle.
    pub fn new() -> Self {
        Self { handle: None }
    }

    /// Opens a serial port at the given path with specified baud rate.
    ///
    /// Uses 10ms read timeout for responsive behavior without CPU spinning.
    pub fn open(mut self, path: &Path, baud_rate: u32) -> Result<Self, SerialError> {
        self.handle = Some(
            serialport::new(path.to_string_lossy(), baud_rate)
                .timeout(Duration::from_millis(10))
                .open()?,
        );
        Ok(self)
    }

    /// Closes the serial port by dropping the handle.
    #[allow(dead_code)]
    pub fn close(&mut self) {
        self.handle = None;
    }

    /// Returns `true` if the port is currently open.
    #[allow(dead_code)]
    pub fn is_open(&self) -> bool {
        self.handle.is_some()
    }

    /// Writes all bytes to the serial port and flushes.
    ///
    /// Returns `NoHandle` error if the port is closed.
    pub fn write_all(&mut self, data: &[u8]) -> Result<(), SerialError> {
        match &mut self.handle {
            Some(port) => {
                port.write_all(data).map_err(SerialError::Write)?;
                port.flush().map_err(SerialError::Write)?;
                Ok(())
            }
            None => Err(SerialError::NoHandle),
        }
    }

    #[allow(dead_code)]
    pub fn device_name(&self) -> Option<String> {
        self.handle.as_ref().and_then(|ser| ser.name())
    }

    /// Reads bytes from the serial port into the buffer.
    ///
    /// Returns the number of bytes read. Returns `0` on timeout (not an error).
    /// Returns `NoHandle` error if the port is closed.
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, SerialError> {
        match &mut self.handle {
            Some(port) => match port.read(buf) {
                Ok(size) => Ok(size),
                Err(e) if e.kind() == std::io::ErrorKind::TimedOut => Ok(0),
                Err(e) => Err(SerialError::Read(e)),
            },
            None => Err(SerialError::NoHandle),
        }
    }

    /// Creates a clone of this handle for separate read/write operations.
    ///
    /// Both handles share the same underlying port. Useful for having
    /// separate reader and writer threads.
    pub fn try_clone(&self) -> Result<Self, SerialError> {
        match &self.handle {
            Some(port) => Ok(PortHandle {
                handle: Some(port.try_clone()?),
            }),
            None => Err(SerialError::NoHandle),
        }
    }
}
