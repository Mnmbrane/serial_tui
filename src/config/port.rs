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
    Lf,
    /// Carriage Return (`\r`)
    Cr,
    /// Carriage Return + Line Feed (`\r\n`)
    CrLf,
}

impl LineEnding {
    /// Returns the byte representation of this line ending.
    pub fn as_bytes(&self) -> &'static [u8] {
        match self {
            LineEnding::Lf => b"\n",
            LineEnding::Cr => b"\r",
            LineEnding::CrLf => b"\r\n",
        }
    }

    pub fn len(&self) -> usize {
        self.as_bytes().len()
    }
}

impl std::str::FromStr for LineEnding {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "lf" | "\n" => Ok(LineEnding::Lf),
            "cr" | "\r" => Ok(LineEnding::Cr),
            "crlf" | "\r\n" => Ok(LineEnding::CrLf),
            other => Err(ConfigError::InvalidLineEnding(other.into())),
        }
    }
}

/// Configuration for a single serial port connection.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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
                line_ending: LineEnding::Lf,
                color: Color(RatatuiColor::Reset),
            }
        );
    }

    #[test]
    fn test_modify() {
        let mut port_config = PortConfig::default();
        port_config.baud_rate = 9600;
        port_config.line_ending = LineEnding::CrLf;
        assert_eq!(port_config.baud_rate, 9600);
        assert_eq!(port_config.line_ending, LineEnding::CrLf);
    }
}
