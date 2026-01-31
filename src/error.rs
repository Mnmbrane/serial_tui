//! Configuration and parsing error types.

use thiserror::Error;

/// Errors from parsing configuration values.
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("invalid color: {0}")]
    InvalidColor(String),

    #[error("invalid line ending: {0}")]
    InvalidLineEnding(String),

    #[error("invalid hex value: {0}")]
    ParseInt(#[from] std::num::ParseIntError),
}
