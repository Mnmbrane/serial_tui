//! Serial-specific error types.

use std::sync::Arc;
use thiserror::Error;

/// Errors that can occur during serial port operations.
#[derive(Debug, Error)]
pub enum SerialError {
    #[error("port not found: {0}")]
    PortNotFound(String),

    #[error("failed to open port: {0}")]
    Open(#[from] serialport::Error),

    #[error("read error: {0}")]
    Read(std::io::Error),

    #[error("write error: {0}")]
    Write(#[source] std::io::Error),

    #[error("failed to send to port")]
    Send(#[from] std::sync::mpsc::SendError<Arc<Vec<u8>>>),
}
