use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
        match value.as_str() {
            "\n" => Ok(LineEnding::LF),
            "\r" => Ok(LineEnding::CR),
            "\r\n" => Ok(LineEnding::CRLF),
            e => Err(format!(
                "Unable to parse line ending {e}, must be \"\\n\", \"\\r\" or \"\\r\\n\"."
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[serde(try_from = "u8")]
pub enum DataBits {
    Five,
    Six,
    Seven,
    #[default]
    Eight,
}

impl TryFrom<u8> for DataBits {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            5 => Ok(DataBits::Five),
            6 => Ok(DataBits::Six),
            7 => Ok(DataBits::Seven),
            8 => Ok(DataBits::Eight),
            e => Err(format!(
                "Unable to parse data bit {e}, must be 5, 6, 7 or 8."
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[serde(try_from = "u8")]
pub enum StopBits {
    #[default]
    One,
    Two,
}

impl TryFrom<u8> for StopBits {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(StopBits::One),
            2 => Ok(StopBits::Two),
            e => Err(format!("Unable to parse stop bit {e}, must be 1 or 2.")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[serde(try_from = "String")]
pub enum Parity {
    #[default]
    None,
    Odd,
    Even,
}

impl TryFrom<String> for Parity {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "none" => Ok(Parity::None),
            "odd" => Ok(Parity::Odd),
            "even" => Ok(Parity::Even),
            e => Err(format!(
                "Unable to parse parity {e}, must be \"none\", \"odd\" or \"even\"."
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[serde(try_from = "String")]
pub enum FlowControl {
    #[default]
    None,
    Software,
    Hardware,
}

impl TryFrom<String> for FlowControl {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "none" => Ok(FlowControl::None),
            "software" => Ok(FlowControl::Software),
            "hardware" => Ok(FlowControl::Hardware),
            e => Err(format!(
                "Unable to parse flow_control {e}, must be \"none\", \"software\" or \"hardware\"."
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[serde(try_from = "String")]
pub enum Color {
    #[default]
    Reset,
    Black,
    DarkGrey,
    Red,
    DarkRed,
    Green,
    DarkGreen,
    Yellow,
    DarkYellow,
    Blue,
    DarkBlue,
    Magenta,
    DarkMagenta,
    Cyan,
    DarkCyan,
    White,
    Grey,
    Rgb {
        r: u8,
        g: u8,
        b: u8,
    },
}

impl TryFrom<String> for Color {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "reset" => Ok(Color::Reset),
            "black" => Ok(Color::Black),
            "darkgrey" => Ok(Color::DarkGrey),
            "red" => Ok(Color::Red),
            "darkred" => Ok(Color::DarkRed),
            "green" => Ok(Color::Green),
            "darkgreen" => Ok(Color::DarkGreen),
            "yellow" => Ok(Color::Yellow),
            "darkyellow" => Ok(Color::DarkYellow),
            "blue" => Ok(Color::Blue),
            "darkblue" => Ok(Color::DarkBlue),
            "magenta" => Ok(Color::Magenta),
            "darkmagenta" => Ok(Color::DarkMagenta),
            "cyan" => Ok(Color::Cyan),
            "darkcyan" => Ok(Color::DarkCyan),
            "white" => Ok(Color::White),
            "grey" => Ok(Color::Grey),
            s if s.starts_with('#') => {
                let hex = s.trim_start_matches('#');

                if hex.len() != 6 {
                    return Err(format!("Invalid hex color: {}", hex));
                }

                Ok(Color::Rgb {
                    r: u8::from_str_radix(&hex[0..=1], 16)
                        .map_err(|_| format!("Invalid hex for r component: {}", &hex[0..=1]))?,
                    g: u8::from_str_radix(&hex[2..=3], 16)
                        .map_err(|_| format!("Invalid hex for g component: {}", &hex[2..=3]))?,
                    b: u8::from_str_radix(&hex[4..=5], 16)
                        .map_err(|_| format!("Invalid hex for b component: {}", &hex[4..=5]))?,
                })
            }
            e => Err(format!("Unable to parse color {e}.")),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(default)]
pub struct PortConfig {
    pub path: PathBuf,
    pub baud_rate: Option<u32>,
    pub data_bits: Option<DataBits>,
    pub stop_bits: Option<StopBits>,
    pub parity: Option<Parity>,
    pub flow_control: Option<FlowControl>,
    pub line_ending: Option<LineEnding>,
    pub color: Option<Color>,
}

impl Default for PortConfig {
    fn default() -> Self {
        Self {
            path: PathBuf::new(),
            baud_rate: Some(115_200),
            data_bits: Some(DataBits::default()),
            stop_bits: Some(StopBits::default()),
            parity: Some(Parity::default()),
            flow_control: Some(FlowControl::default()),
            line_ending: Some(LineEnding::default()),
            color: Some(Color::default()),
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
                baud_rate: Some(115_200),
                data_bits: Some(DataBits::Eight),
                stop_bits: Some(StopBits::One),
                parity: Some(Parity::None),
                flow_control: Some(FlowControl::None),
                line_ending: Some(LineEnding::LF),
                color: Some(Color::Reset),
            }
        );
    }

    #[test]
    fn test_example() {
        let mut port_config: PortConfig = PortConfig::default();
        port_config.baud_rate = Some(9600);
        port_config.line_ending = Some(LineEnding::CRLF);
        assert_eq!(port_config.baud_rate, Some(9600));
        assert_eq!(port_config.line_ending, Some(LineEnding::CRLF));
    }
}
