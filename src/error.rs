use std::num::ParseIntError;

use serde::{Deserializer, ser};

#[derive(Debug)]
pub enum AppError {
    Io(std::io::Error),
    InvalidSerialize(String),
    InvalidDeserialize(String),
    InvalidPortName(&'static str),
    InvalidFilePath(&'static str),
    ConfigPortInsert(&'static str),
    InvalidDataBits(&'static str),
    InvalidColor(&'static str),
    ParseIntError(ParseIntError),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use AppError::*;
        match self {
            Io(e) => write!(f, ""),
            InvalidDeserialize(e) => write!(f, "Toml Deserialize error: {e}"),
            InvalidSerialize(e) => write!(f, "IO error: {e}"),
            InvalidPortName(e) => write!(f, "Invalid Port Name: {e}"),
            InvalidFilePath(e) => write!(f, "Invalid File Path: {e}"),
            ConfigPortInsert(e) => write!(f, "Could not insert new port element: {e}"),
            InvalidDataBits(e) => write!(f, "Invalid data bits: {e}"),
            InvalidColor(e) => write!(f, "Invalid color: {e}"),
            ParseIntError(e) => write!(f, "Invalid number: {e}"),
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        AppError::Io(value)
    }
}

impl From<toml::de::Error> for AppError {
    fn from(e: toml::de::Error) -> Self {
        AppError::InvalidDeserialize(format!("Error trying to deserialize: {e}"))
    }
}

impl From<toml::ser::Error> for AppError {
    fn from(e: toml::ser::Error) -> Self {
        AppError::InvalidSerialize(format!("Error trying to serialize: {e}"))
    }
}
