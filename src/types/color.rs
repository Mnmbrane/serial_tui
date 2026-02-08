//! Terminal color wrapper with serde support.

use std::{fmt::Display, str::FromStr};

use ratatui::style::Color as RatatuiColor;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::error::ConfigError;

/// Wrapper around ratatui::Color with custom serialization.
///
/// Supports parsing from:
/// - Hex colors: `"#FF8000"`
/// - Named colors: `"green"`, `"red"`, `"blue"`, etc.
#[derive(Debug, PartialEq, Clone)]
pub struct Color(pub RatatuiColor);

impl FromStr for Color {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Hex color
        if s.starts_with('#') {
            if s.len() != 7 {
                return Err(ConfigError::InvalidColor("hex color must be #RRGGBB".into()));
            }
            let r = u8::from_str_radix(&s[1..3], 16)?;
            let g = u8::from_str_radix(&s[3..5], 16)?;
            let b = u8::from_str_radix(&s[5..7], 16)?;
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
            other => return Err(ConfigError::InvalidColor(other.into())),
        };
        Ok(Color(color))
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            RatatuiColor::Rgb(r, g, b) => write!(f, "#{r:02X}{g:02X}{b:02X}"),
            RatatuiColor::Reset => f.write_str("reset"),
            RatatuiColor::Black => f.write_str("black"),
            RatatuiColor::Red => f.write_str("red"),
            RatatuiColor::Green => f.write_str("green"),
            RatatuiColor::Yellow => f.write_str("yellow"),
            RatatuiColor::Blue => f.write_str("blue"),
            RatatuiColor::Magenta => f.write_str("magenta"),
            RatatuiColor::Cyan => f.write_str("cyan"),
            RatatuiColor::Gray => f.write_str("gray"),
            RatatuiColor::White => f.write_str("white"),
            _ => f.write_str("reset"),
        }
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Color, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Color::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl Serialize for Color {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}
