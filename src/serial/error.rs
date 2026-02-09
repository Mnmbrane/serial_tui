//! Serial-specific error types.

use std::sync::{Arc, mpsc::TrySendError};

use bytes::Bytes;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SerialError {
    #[error("port not found: {0}")]
    PortNotFound(Arc<str>),

    #[error("failed to open port: {0}")]
    Open(#[from] serialport::Error),

    #[error("write channel full or closed")]
    TrySend(#[from] TrySendError<Bytes>),
}
