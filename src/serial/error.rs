//! Serial-specific error types.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum SerialError {
    #[error("port not found: {0}")]
    PortNotFound(String),

    #[error("failed to open port: {0}")]
    Open(#[from] serialport::Error),

    #[error("read error: {0}")]
    Read(std::io::Error),

    #[error("failed to send to port")]
    Send(#[from] std::sync::mpsc::SendError<Vec<u8>>),
}
