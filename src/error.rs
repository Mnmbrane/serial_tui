use std::{num::ParseIntError, sync::PoisonError};

use serde::{Deserializer, ser};

#[derive(Debug)]
pub enum AppError {
    InvalidIO(std::io::Error),
    InvalidSerialize(String),
    InvalidDeserialize(String),
    InvalidPortName(String),
    InvalidFilePath(String),
    ConfigPortInsert(String),
    InvalidDataBits(String),
    InvalidColor(String),
    InvalidLineEnding(String),
    ParseIntError(ParseIntError),
    PoisonError(String),

    PortMapInvalidGet(String),

    PortHandleInvalidOpen(String),
    PortHandleInvalidWrite(String),
    PortHandleInvalidRead(String),

    SerialOpenError(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use AppError::*;
        match self {
            InvalidIO(e) => write!(f, "Invalid IO: {e}"),
            InvalidDeserialize(e) => write!(f, "Toml Deserialize error: {e}"),
            InvalidSerialize(e) => write!(f, "IO error: {e}"),
            InvalidPortName(e) => write!(f, "Invalid Port Name: {e}"),
            InvalidFilePath(e) => write!(f, "Invalid File Path: {e}"),
            ConfigPortInsert(e) => write!(f, "Could not insert new port element: {e}"),
            InvalidDataBits(e) => write!(f, "Invalid data bits: {e}"),
            InvalidColor(e) => write!(f, "Invalid color: {e}"),
            InvalidLineEnding(e) => write!(f, "Invalide Line Ending: {e}"),
            ParseIntError(e) => write!(f, "Invalid number: {e}"),
            PoisonError(e) => write!(f, "LockPoisonError: {e}"),
            PortMapInvalidGet(e) => write!(f, "Port Map Invalid Get: {e}"),

            PortHandleInvalidOpen(e) => write!(f, "Port Map Invalid Open: {e}"),
            PortHandleInvalidWrite(e) => write!(f, "Port Map Invalid Write: {e}"),
            PortHandleInvalidRead(e) => write!(f, "Port Map Invalid Read: {e}"),

            SerialOpenError(e) => write!(f, "Tried to open a serial port: {e}"),
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        AppError::InvalidIO(value)
    }
}

impl<T> From<PoisonError<T>> for AppError {
    fn from(value: PoisonError<T>) -> Self {
        AppError::PoisonError(value.to_string())
    }
}

impl From<toml::de::Error> for AppError {
    fn from(e: toml::de::Error) -> Self {
        AppError::InvalidDeserialize(e.to_string())
    }
}

impl From<toml::ser::Error> for AppError {
    fn from(e: toml::ser::Error) -> Self {
        AppError::InvalidSerialize(e.to_string())
    }
}
