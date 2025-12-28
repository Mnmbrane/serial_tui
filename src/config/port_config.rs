use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub enum LineEnding {
    None,
    #[default]
    LF,
    CR,
    CRLF,
}

impl From<&str> for LineEnding {
    fn from(value: &str) -> Self {
        match value {
            "\n" => LineEnding::LF,
            "\r" => LineEnding::CR,
            "\r\n" => LineEnding::CRLF,
            _ => LineEnding::None,
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub enum DataBits {
    Five,
    Six,
    Seven,
    #[default]
    Eight,
}

#[derive(Serialize, Deserialize, Default, PartialEq, Debug)]
pub enum StopBits {
    #[default]
    One,
    Two,
}

#[derive(Serialize, Deserialize, Default, PartialEq, Debug)]
pub enum Parity {
    #[default]
    None,
    Odd,
    Even,
}

#[derive(Serialize, Deserialize, Default, PartialEq, Debug)]
pub enum FlowControl {
    #[default]
    None,
    Software,
    Hardware,
}

#[derive(Serialize, Deserialize, Default, PartialEq, Debug)]
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
    AnsiValue(u8),
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(default)]
pub struct PortConfig {
    pub path: PathBuf,
    pub baud_rate: u32,
    pub data_bits: DataBits,
    pub stop_bits: StopBits,
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
            data_bits: DataBits::default(),
            stop_bits: StopBits::default(),
            parity: Parity::default(),
            flow_control: FlowControl::default(),
            line_ending: LineEnding::default(),
            color: Color::default(),
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
                data_bits: DataBits::Eight,
                stop_bits: StopBits::One,
                parity: Parity::None,
                flow_control: FlowControl::None,
                line_ending: LineEnding::LF,
                color: Color::Reset,
            }
        );
    }

    #[test]
    fn test_diff_baud_and_le() {
        let mut port_config: PortConfig = PortConfig::default();
        port_config.baud_rate = 9600;
        port_config.line_ending = LineEnding::CRLF;
        assert_eq!(port_config.baud_rate, 9600);
        assert_eq!(port_config.line_ending, "\r\n".into());
    }
}
