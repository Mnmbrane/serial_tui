//! Serial-specific error types.

use std::sync::Arc;

use bytes::Bytes;
use thiserror::Error;
use tokio::sync::mpsc::error::TrySendError;

#[derive(Debug, Error)]
pub enum SerialError {
    #[error("port not found: {0}")]
    PortNotFound(Arc<str>),

    #[error("failed to open port: {0}")]
    Open(#[from] serialport::Error),

    #[error("read error: {0}")]
    Read(std::io::Error),

    #[error("write channel full or closed")]
    TrySend(#[from] TrySendError<Bytes>),
}
