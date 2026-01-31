//! Terminal color wrapper with serde support.

use std::{fmt::Display, str::FromStr};

use ratatui::style::Color as RatatuiColor;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::error::AppError;

/// Wrapper around ratatui::Color with custom serialization.
///
/// Supports parsing from:
/// - Hex colors: `"#FF8000"`
/// - Named colors: `"green"`, `"red"`, `"blue"`, etc.
///
/// Serializes RGB colors as hex, named colors as lowercase strings.
#[derive(Debug, PartialEq, Clone)]
pub struct Color(pub RatatuiColor);

impl std::str::FromStr for Color {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Hex color
        if s.starts_with('#') {
            if s.len() != 7 {
                return Err(AppError::InvalidColor("hex color must be #RRGGBB".into()));
            }
            let r = u8::from_str_radix(&s[1..3], 16).map_err(AppError::ParseIntError)?;
            let g = u8::from_str_radix(&s[3..5], 16).map_err(AppError::ParseIntError)?;
            let b = u8::from_str_radix(&s[5..7], 16).map_err(AppError::ParseIntError)?;
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
            _ => return Err(AppError::InvalidColor("unknown color '{}'".into())),
        };
        Ok(Color(color))
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self.0 {
            RatatuiColor::Rgb(r, g, b) => &format!("#{r:02X}{g:02X}{b:02X}"),
            RatatuiColor::Reset => "reset",
            RatatuiColor::Black => "black",
            RatatuiColor::Red => "red",
            RatatuiColor::Green => "green",
            RatatuiColor::Yellow => "yellow",
            RatatuiColor::Blue => "blue",
            RatatuiColor::Magenta => "magenta",
            RatatuiColor::Cyan => "cyan",
            RatatuiColor::Gray => "gray",
            RatatuiColor::White => "white",
            _ => "reset", // fallback
        };

        write!(f, "{s}")
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Color, D::Error>
    where
        D: Deserializer<'de>,
    {
        let color = String::deserialize(deserializer)?;
        let color = color.as_str();

        Color::from_str(color).map_err(serde::de::Error::custom)
    }
}

impl Serialize for Color {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.to_string().as_str())
    }
}
