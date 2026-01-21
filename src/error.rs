//! Application-wide error types.
//!
//! Uses `thiserror` for deriving `std::error::Error` implementations.
//! All errors across the app are consolidated into `AppError` for
//! unified error handling with the `?` operator.

use core::error;
use std::{io::Error, num::ParseIntError, sync::mpsc::SendError};

use thiserror::Error;

/// Central error type for the application.
///
/// Wraps errors from IO, serialization, serial ports, and internal logic.
/// Implements `From` for common error types to enable `?` propagation.
#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    InvalidIO(#[from] std::io::Error),
    #[error("No port handle")]
    NoPortHandleError,
    #[error("Toml serial error: {0}")]
    InvalidSerialize(#[from] toml::ser::Error),
    #[error("Toml deserial error: {0}")]
    InvalidDeserialize(#[from] toml::de::Error),
    #[error("Invalid Color: {0}")]
    InvalidColor(String),
    #[error("Invalid Line ending : {0}")]
    InvalidLineEnding(String),
    #[error(transparent)]
    ParseIntError(ParseIntError),
    #[error("Serial port error: {0}")]
    SerialPortError(#[from] serialport::Error),
    #[error("Serial port read error: {0}")]
    SerialPortReadError(Error),
    #[error("Toml file save error")]
    PortConfigSaveError,
    #[error("Cloning port failed")]
    PortCloneFailed,
    #[error("Lock Poisoned")]
    LockPoisoned,
    #[error("Invalid Send: {0}")]
    InvalidSend(SendError<std::sync::Arc<Vec<u8>>>),
    #[error("Invalid Map Key")]
    InvalidMapKey,
}
