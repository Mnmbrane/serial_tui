//! Low-level serial port handle wrapper.

use std::{path::Path, time::Duration};

use serialport::SerialPort;

use super::SerialError;

/// Low-level wrapper around a serial port.
#[derive(Default)]
pub struct Handle {
    inner: Option<Box<dyn SerialPort>>,
}

impl Handle {
    /// Opens a serial port at the given path with specified baud rate.
    pub fn open(path: &Path, baud_rate: u32) -> Result<Self, SerialError> {
        let port = serialport::new(path.to_string_lossy(), baud_rate)
            .timeout(Duration::from_millis(10))
            .open()?;
        Ok(Self { inner: Some(port) })
    }

    /// Closes the serial port.
    #[allow(dead_code)]
    pub fn close(&mut self) {
        self.inner = None;
    }

    /// Returns `true` if the port is open.
    #[allow(dead_code)]
    pub fn is_open(&self) -> bool {
        self.inner.is_some()
    }

    /// Writes all bytes and flushes.
    pub fn write_all(&mut self, data: &[u8]) -> Result<(), SerialError> {
        match &mut self.inner {
            Some(port) => {
                port.write_all(data).map_err(SerialError::Write)?;
                port.flush().map_err(SerialError::Write)?;
                Ok(())
            }
            None => Err(SerialError::NoHandle),
        }
    }

    /// Returns the device name if available.
    #[allow(dead_code)]
    pub fn device_name(&self) -> Option<String> {
        self.inner.as_ref().and_then(|p| p.name())
    }

    /// Reads bytes into the buffer. Returns 0 on timeout.
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, SerialError> {
        match &mut self.inner {
            Some(port) => match port.read(buf) {
                Ok(n) => Ok(n),
                Err(e) if e.kind() == std::io::ErrorKind::TimedOut => Ok(0),
                Err(e) => Err(SerialError::Read(e)),
            },
            None => Err(SerialError::NoHandle),
        }
    }

    /// Creates a clone for separate read/write operations.
    pub fn try_clone(&self) -> Result<Self, SerialError> {
        match &self.inner {
            Some(port) => Ok(Handle {
                inner: Some(port.try_clone()?),
            }),
            None => Err(SerialError::NoHandle),
        }
    }
}
