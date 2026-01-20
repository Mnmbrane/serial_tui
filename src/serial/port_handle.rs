use std::{path::PathBuf, time::Duration};

use serialport::SerialPort;

use crate::error::AppError;

// Port handle contains connection info
// about the serial port
#[derive(Default)]
pub struct PortHandle {
    handle: Option<Box<dyn SerialPort>>,
}

impl PortHandle {
    pub fn new() -> Self {
        Self { handle: None }
    }

    /// Open serial port and set handle
    pub fn open(mut self, path: &PathBuf, baud_rate: u32) -> Result<Self, AppError> {
        // Open a port
        self.handle = Some(
            serialport::new(path.to_string_lossy(), baud_rate)
                .timeout(Duration::from_millis(1000))
                .open()
                .expect("Failed to open port"),
        );
        Ok(self)
    }

    /// Close serial port
    pub fn close(&mut self) {
        self.handle = None; // Drop and closes the port
    }

    /// Check if open
    /// @return false if not open
    pub fn is_open(&self) -> bool {
        self.handle.is_some()
    }

    pub fn write_all(&mut self, data: &[u8]) -> Result<(), AppError> {
        match &mut self.handle {
            Some(port) => port.write_all(data).map_err(|e| AppError::InvalidIO(e)),
            None => Err(AppError::NoPortHandleError),
        }
    }

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
