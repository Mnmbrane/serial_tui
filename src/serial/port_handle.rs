//! Low-level serial port handle wrapper.
//!
//! Wraps the `serialport` crate's `SerialPort` trait object,
//! providing a simpler interface for open/close/read/write operations.

use std::{path::PathBuf, time::Duration};

use serialport::SerialPort;

use crate::error::AppError;

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
    /// Sets a 1 second read timeout. Consumes self and returns a new
    /// handle with the open port, or an error if the port couldn't be opened.
    pub fn open(mut self, path: &PathBuf, baud_rate: u32) -> Result<Self, AppError> {
        self.handle = Some(
            serialport::new(path.to_string_lossy(), baud_rate)
                .timeout(Duration::from_millis(1000))
                .open()?,
        );
        Ok(self)
    }

    /// Closes the serial port by dropping the handle.
    pub fn close(&mut self) {
        self.handle = None;
    }

    /// Returns `true` if the port is currently open.
    pub fn is_open(&self) -> bool {
        self.handle.is_some()
    }

    /// Writes all bytes to the serial port.
    ///
    /// Returns `NoPortHandleError` if the port is closed.
    pub fn write_all(&mut self, data: &[u8]) -> Result<(), AppError> {
        match &mut self.handle {
            Some(port) => port.write_all(data).map_err(|e| AppError::InvalidIO(e)),
            None => Err(AppError::NoPortHandleError),
        }
    }

    /// Reads bytes from the serial port into the buffer.
    ///
    /// Returns the number of bytes read. Returns `0` on timeout (not an error).
    /// Returns `NoPortHandleError` if the port is closed.
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, AppError> {
        match &mut self.handle {
            Some(port) => match port.read(buf) {
                Ok(size) => Ok(size),
                Err(e) if e.kind() == std::io::ErrorKind::TimedOut => Ok(0),
                Err(e) => Err(AppError::SerialPortReadError(e)),
            },
            None => Err(AppError::NoPortHandleError),
        }
    }

    /// Creates a clone of this handle for separate read/write operations.
    ///
    /// Both handles share the same underlying port. Useful for having
    /// separate reader and writer threads.
    pub fn try_clone(&self) -> Result<Self, AppError> {
        if self.handle.is_none() {
            return Err(AppError::NoPortHandleError);
        }

        match &self.handle {
            Some(clone) => Ok(PortHandle {
                handle: Some(clone.try_clone()?),
            }),
            None => Ok(PortHandle { handle: None }),
        }
    }
}
