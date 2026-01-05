use std::time::Duration;

use serialport::SerialPort;

use crate::{error::AppError, serial::PortInfo};

// Port handle contains connection info
// about the serial port
pub struct PortHandle {
    port_info: PortInfo,
    /// Serial ports to read and write data to
    connection: Option<Box<dyn SerialPort>>,
}

impl PortHandle {
    pub fn new(port_info: PortInfo) -> Self {
        Self {
            port_info,
            connection: None,
        }
    }

    pub fn open(&mut self) -> Result<(), AppError> {
        self.connection = Some(
            serialport::new(
                self.port_info.path.to_string_lossy(),
                self.port_info.baud_rate,
            )
            .timeout(Duration::from_millis(100))
            .open()
            .map_err(|e| AppError::PortHandleInvalidOpen(e.to_string()))?,
        );
        Ok(())
    }

    pub fn close(&mut self) {
        self.connection = None; // Drop and closes the port
    }

    pub fn is_open(&self) -> bool {
        self.connection.is_some()
    }

    pub fn write(&mut self, data: &[u8]) -> Result<usize, AppError> {
        match &mut self.connection {
            Some(port) => port
                .write(data)
                .map_err(|e| AppError::PortHandleInvalidWrite(e.to_string())),
            None => Err(AppError::PortHandleInvalidWrite(format!(
                "No connection for {}",
                self.port_info.path.to_string_lossy()
            ))),
        }
    }

    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, AppError> {
        match &mut self.connection {
            Some(port) => port
                .read(buf)
                .map_err(|e| AppError::PortHandleInvalidRead(e.to_string())),
            None => Err(AppError::PortHandleInvalidRead(format!(
                "No connection for {}",
                self.port_info.path.to_string_lossy()
            ))),
        }
    }
}
