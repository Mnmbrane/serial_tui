pub mod port_config;
pub use port_config::PortConfig;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs};

use crate::error::AppError;

// Want just 2 differnt configs for now.
// 1. PortConfig - Contains com port details
// 2. MacroConfig - Contains keybindings for VIM Motions (TODO)
#[derive(Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(flatten)]
    port_config: HashMap<String, PortConfig>,
}

impl AppConfig {
    pub fn new() -> Self {
        Self {
            port_config: HashMap::new(),
        }
    }

    pub fn init(&mut self, port_cfg_path: &str) -> Result<(), AppError> {
        self.port_config = toml::from_str(port_cfg_path)?;
        Ok(())
    }

    pub fn save_port_cfg(&self, port_cfg_path: &str) -> Result<(), AppError> {
        let content = toml::to_string_pretty(&self.port_config)?;
        fs::write(port_cfg_path, content)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::port_config::{Color, DataBits, FlowControl, LineEnding, Parity, StopBits};
    use std::path::PathBuf;

    #[test]
    fn test_parse_valid_config() {
        let mut app_config = AppConfig::new();
        app_config
            .init(include_str!("test/valid_config.toml"))
            .unwrap();

        assert_eq!(
            app_config.port_config.get("port1").unwrap(),
            &PortConfig {
                path: PathBuf::from("/dev/ttyUSB0"),
                baud_rate: 115200,
                data_bits: DataBits::Eight,
                stop_bits: StopBits::One,
                parity: Parity::None,
                flow_control: FlowControl::None,
                line_ending: LineEnding::CRLF,
                color: Color::Green,
            }
        );

        assert_eq!(
            app_config.port_config.get("port2").unwrap(),
            &PortConfig {
                path: PathBuf::from("/dev/ttyUSB1"),
                baud_rate: 9600,
                data_bits: DataBits::Seven,
                stop_bits: StopBits::Two,
                parity: Parity::Even,
                flow_control: FlowControl::Hardware,
                line_ending: LineEnding::LF,
                color: Color::Rgb {
                    r: 255,
                    g: 128,
                    b: 0
                },
            }
        );
    }

    #[test]
    fn test_parse_minimal_uses_defaults() {
        let mut app_config = AppConfig::new();
        app_config
            .init(include_str!("test/minimal_config.toml"))
            .unwrap();

        assert_eq!(
            app_config.port_config.get("port1").unwrap(),
            &PortConfig {
                path: PathBuf::from("/dev/ttyUSB0"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn test_invalid_baud_fails() {
        let mut app_config = AppConfig::new();
        assert!(
            app_config
                .init(include_str!("test/invalid_baud.toml"))
                .is_err()
        );
    }

    #[test]
    fn test_invalid_databits_fails() {
        let mut app_config = AppConfig::new();
        assert!(
            app_config
                .init(include_str!("test/invalid_databits.toml"))
                .is_err()
        );
    }

    #[test]
    fn test_invalid_color_fails() {
        let mut app_config = AppConfig::new();
        assert!(
            app_config
                .init(include_str!("test/invalid_color.toml"))
                .is_err()
        );
    }

    #[test]
    fn test_line_ending_parsing() {
        assert_eq!(LineEnding::try_from("lf".to_string()), Ok(LineEnding::LF));
        assert_eq!(LineEnding::try_from("cr".to_string()), Ok(LineEnding::CR));
        assert_eq!(
            LineEnding::try_from("crlf".to_string()),
            Ok(LineEnding::CRLF)
        );
        assert!(LineEnding::try_from("invalid".to_string()).is_err());
    }

    #[test]
    fn test_color_hex_parsing() {
        assert_eq!(
            Color::try_from("#ff8000".to_string()),
            Ok(Color::Rgb {
                r: 255,
                g: 128,
                b: 0
            })
        );
        assert!(Color::try_from("#fff".to_string()).is_err());
        assert!(Color::try_from("#gggggg".to_string()).is_err());
    }
}
