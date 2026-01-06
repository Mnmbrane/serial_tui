use std::{path::PathBuf, time::Duration};

use serialport::SerialPort;

use crate::error::AppError;

// Port handle contains connection info
// about the serial port

pub struct PortHandle {
    handle: Option<Box<dyn SerialPort>>,
}

impl PortHandle {
    pub fn new() -> Self {
        Self { handle: None }
    }

    pub fn open(&mut self, path: PathBuf, baud_rate: u32) -> Result<(), AppError> {
        serialport::new(path.to_string_lossy(), baud_rate)
            .timeout(Duration::MAX)
            .open()
            .map_err(|e| AppError::PortHandleInvalidOpen(e.to_string()))?;

        Ok(())
    }

    pub fn close(&mut self) {
        self.handle = None; // Drop and closes the port
    }

    pub fn is_open(&self) -> bool {
        self.handle.is_some()
    }

    pub fn write(&mut self, data: &[u8]) -> Result<usize, AppError> {
        match &mut self.handle {
            Some(port) => port
                .write(data)
                .map_err(|e| AppError::PortHandleInvalidWrite(e.to_string())),
            None => Err(AppError::PortHandleInvalidWrite(format!("No connection",))),
        }
    }

    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, AppError> {
        match &mut self.handle {
            Some(port) => port
                .read(buf)
                .map_err(|e| AppError::PortHandleInvalidRead(e.to_string())),
            None => Err(AppError::PortHandleInvalidRead(format!("No connection",))),
        }
    }
}
