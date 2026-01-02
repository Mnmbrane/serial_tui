use ratatui::style::Color as RatatuiColor;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error};
use serialport::{FlowControl, Parity};
use std::{path::PathBuf, str::FromStr};

use crate::error::AppError;

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

#[derive(Debug, PartialEq, Clone)]
pub struct Color(pub RatatuiColor);

impl FromStr for Color {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Hex color
        if s.starts_with('#') {
            if s.len() != 7 {
                return Err(AppError::InvalidColor("hex color must be #RRGGBB"));
            }
            let r = u8::from_str_radix(&s[1..3], 16).map_err(|e| AppError::ParseIntError(e))?;
            let g = u8::from_str_radix(&s[3..5], 16).map_err(|e| AppError::ParseIntError(e))?;
            let b = u8::from_str_radix(&s[5..7], 16).map_err(|e| AppError::ParseIntError(e))?;
            return Ok(Color(RatatuiColor::Rgb(r, g, b)));
        }

        // Named color
        let color = match s.to_lowercase().as_str() {
            "reset" => RatatuiColor::Reset,
            "black" => RatatuiColor::Black,
            "red" => RatatuiColor::Red,
            "green" => RatatuiColor::Green,
            "yellow" => RatatuiColor::Yellow,
            "blue" => RatatuiColor::Blue,
            "magenta" => RatatuiColor::Magenta,
            "cyan" => RatatuiColor::Cyan,
            "gray" | "grey" => RatatuiColor::Gray,
            "white" => RatatuiColor::White,
            _ => return Err(AppError::InvalidColor("unknown color '{}'")),
        };
        Ok(Color(color))
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Color, D::Error>
    where
        D: Deserializer<'de>,
    {
        let color: String = String::deserialize(deserializer)?;

        // Parse hex color
        if color.starts_with('#') {
            if color.len() != 7 {
                return Err(D::Error::custom("hex color must be #RRGGBB"));
            }

            let r = u8::from_str_radix(&color[1..3], 16)
                .map_err(|_| D::Error::custom(format!("invalid hex '{}'", color)))?;
            let g = u8::from_str_radix(&color[3..5], 16)
                .map_err(|_| D::Error::custom(format!("invalid hex '{}'", color)))?;
            let b = u8::from_str_radix(&color[5..7], 16)
                .map_err(|_| D::Error::custom(format!("invalid hex '{}'", color)))?;

            return Ok(Color(RatatuiColor::Rgb(r, g, b)));
        }

        // Parse named color
        let color = match color.to_lowercase().as_str() {
            "reset" => RatatuiColor::Reset,
            "black" => RatatuiColor::Black,
            "red" => RatatuiColor::Red,
            "green" => RatatuiColor::Green,
            "yellow" => RatatuiColor::Yellow,
            "blue" => RatatuiColor::Blue,
            "magenta" => RatatuiColor::Magenta,
            "cyan" => RatatuiColor::Cyan,
            "gray" | "grey" => RatatuiColor::Gray,
            "white" => RatatuiColor::White,
            _ => {
                return Err(D::Error::custom(format!(
                    "invalid color '{}', must be #RRGGBB or a color name",
                    color
                )));
            }
        };

        Ok(Color(color))
    }
}

impl Serialize for Color {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self.0 {
            RatatuiColor::Rgb(r, g, b) => {
                serializer.serialize_str(&format!("#{:02X}{:02X}{:02X}", r, g, b))
            }
            RatatuiColor::Reset => serializer.serialize_str("reset"),
            RatatuiColor::Black => serializer.serialize_str("black"),
            RatatuiColor::Red => serializer.serialize_str("red"),
            RatatuiColor::Green => serializer.serialize_str("green"),
            RatatuiColor::Yellow => serializer.serialize_str("yellow"),
            RatatuiColor::Blue => serializer.serialize_str("blue"),
            RatatuiColor::Magenta => serializer.serialize_str("magenta"),
            RatatuiColor::Cyan => serializer.serialize_str("cyan"),
            RatatuiColor::Gray => serializer.serialize_str("gray"),
            RatatuiColor::White => serializer.serialize_str("white"),
            _ => serializer.serialize_str("reset"), // fallback
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
