//! Serial port configuration.

use ratatui::style::Color as RatatuiColor;
use serde::{Deserialize, Serialize};
use serialport::SerialPort;
use std::path::PathBuf;

use crate::{error::AppError, types::color::Color};

/// Line ending style for serial communication.
#[serde(try_from = "String")]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum LineEnding {
    /// Line Feed (`\n`)
    #[default]
    LF,
    /// Carriage Return (`\r`)
    CR,
    /// Carriage Return + Line Feed (`\r\n`)
    CRLF,
}

impl TryFrom<String> for LineEnding {
    type Error = AppError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "lf" | "\n" => Ok(LineEnding::LF),
            "cr" | "\r" => Ok(LineEnding::CR),
            "crlf" | "\r\n" => Ok(LineEnding::CRLF),
            e => Err(AppError::InvalidLineEnding(e.into())),
        }
    }
}

/// Configuration for a single serial port connection.
///
/// Contains all parameters needed to open and communicate with a serial device,
/// plus display settings like color for the TUI.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(default)]
pub struct PortInfo {
    /// Device path (e.g., "/dev/ttyUSB0", "COM3")
    pub path: PathBuf,
    /// Baud rate in bits per second
    pub baud_rate: u32,
    /// Line ending style for transmitted data
    pub line_ending: LineEnding,
    /// Display color for this port's output in the TUI
    pub color: Color,
}

impl Default for PortInfo {
    fn default() -> Self {
        Self {
            path: PathBuf::new(),
            baud_rate: 115_200,
            line_ending: LineEnding::default(),
            color: Color(RatatuiColor::Reset),
        }
    }
}

// Unit tests
#[cfg(test)] // Only compiels the module during testing
mod tests {
    use crate::types::color::Color;

    use super::*;

    #[test]
    fn test_new() {
        let port_config: PortInfo = PortInfo::default();
        assert_eq!(
            port_config,
            PortInfo {
                path: PathBuf::new(),
                baud_rate: 115_200,
                line_ending: LineEnding::LF,
                color: Color(RatatuiColor::Reset),
            }
        );
    }

    #[test]
    fn test_example() {
        let mut port_config: PortInfo = PortInfo::default();
        port_config.baud_rate = 9600;
        port_config.line_ending = LineEnding::CRLF;
        assert_eq!(port_config.baud_rate, 9600);
        assert_eq!(port_config.line_ending, LineEnding::CRLF);
    }
}
