use ratatui::style::Color as RatatuiColor;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error};
use serialport::{FlowControl, Parity};
use std::{path::PathBuf, str::FromStr};

use crate::{error::AppError, types::Color};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[serde(try_from = "String")]
pub enum LineEnding {
    #[default]
    LF,
    CR,
    CRLF,
}

impl TryFrom<String> for LineEnding {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "lf" | "\n" => Ok(LineEnding::LF),
            "cr" | "\r" => Ok(LineEnding::CR),
            "crlf" | "\r\n" => Ok(LineEnding::CRLF),
            e => Err(format!(
                "Unable to parse line ending {e}, must be \"\\n\", \"\\r\", \"\\r\\n\", \"lf\", \"cr\", or \"crlf\"."
            )),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(default)]
pub struct PortConfig {
    pub path: PathBuf,
    pub baud_rate: u32,
    pub data_bits: u8,
    pub stop_bits: u8,
    pub parity: Parity,
    pub flow_control: FlowControl,
    pub line_ending: LineEnding,
    pub color: Color,
}

impl Default for PortConfig {
    fn default() -> Self {
        Self {
            path: PathBuf::new(),
            baud_rate: 115_200,
            data_bits: 8,
            stop_bits: 1,
            parity: Parity::None,
            flow_control: FlowControl::None,
            line_ending: LineEnding::default(),
            color: Color(RatatuiColor::Reset),
        }
    }
}

// Unit tests
#[cfg(test)] // Only compiels the module during testing
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let port_config: PortConfig = PortConfig::default();
        assert_eq!(
            port_config,
            PortConfig {
                path: PathBuf::new(),
                baud_rate: 115_200,
                data_bits: 8,
                stop_bits: 1,
                parity: Parity::None,
                flow_control: FlowControl::None,
                line_ending: LineEnding::LF,
                color: Color(RatatuiColor::Reset),
            }
        );
    }

    #[test]
    fn test_example() {
        let mut port_config: PortConfig = PortConfig::default();
        port_config.baud_rate = 9600;
        port_config.line_ending = LineEnding::CRLF;
        assert_eq!(port_config.baud_rate, 9600);
        assert_eq!(port_config.line_ending, LineEnding::CRLF);
    }
}
