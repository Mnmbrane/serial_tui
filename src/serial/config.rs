//! Serial port configuration.

use ratatui::style::Color as RatatuiColor;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::{error::ConfigError, types::color::Color};

/// Line ending style for serial communication.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[allow(clippy::upper_case_acronyms)]
pub enum LineEnding {
    /// Line Feed (`\n`)
    #[default]
    LF,
    /// Carriage Return (`\r`)
    CR,
    /// Carriage Return + Line Feed (`\r\n`)
    CRLF,
}

impl LineEnding {
    /// Returns the byte representation of this line ending.
    pub fn as_bytes(&self) -> &'static [u8] {
        match self {
            LineEnding::LF => b"\n",
            LineEnding::CR => b"\r",
            LineEnding::CRLF => b"\r\n",
        }
    }
}

impl TryFrom<String> for LineEnding {
    type Error = ConfigError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "lf" | "\n" => Ok(LineEnding::LF),
            "cr" | "\r" => Ok(LineEnding::CR),
            "crlf" | "\r\n" => Ok(LineEnding::CRLF),
            other => Err(ConfigError::InvalidLineEnding(other.into())),
        }
    }
}

/// Configuration for a single serial port connection.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct PortConfig {
    /// Device path (e.g., "/dev/ttyUSB0", "COM3")
    pub path: PathBuf,
    /// Baud rate in bits per second
    pub baud_rate: u32,
    /// Line ending style for transmitted data
    pub line_ending: LineEnding,
    /// Display color for this port's output in the TUI
    pub color: Color,
}

impl PartialEq for PortConfig {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
            && self.baud_rate == other.baud_rate
            && self.line_ending == other.line_ending
            && self.color == other.color
    }
}

impl Default for PortConfig {
    fn default() -> Self {
        Self {
            path: PathBuf::new(),
            baud_rate: 115_200,
            line_ending: LineEnding::default(),
            color: Color(RatatuiColor::Reset),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let port_config = PortConfig::default();
        assert_eq!(
            port_config,
            PortConfig {
                path: PathBuf::new(),
                baud_rate: 115_200,
                line_ending: LineEnding::LF,
                color: Color(RatatuiColor::Reset),
            }
        );
    }

    #[test]
    fn test_modify() {
        let mut port_config = PortConfig::default();
        port_config.baud_rate = 9600;
        port_config.line_ending = LineEnding::CRLF;
        assert_eq!(port_config.baud_rate, 9600);
        assert_eq!(port_config.line_ending, LineEnding::CRLF);
    }
}
