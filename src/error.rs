use std::{io::Error, num::ParseIntError};

use thiserror::Error;

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
}
