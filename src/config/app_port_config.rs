use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::Display,
    fs::{self, read_to_string},
    path::Path,
};

use crate::{config::PortConfig, error::AppError};

// Want just 2 differnt configs for now.
// 1. PortConfig - Contains com port details
// 2. MacroConfig - Contains keybindings for VIM Motions (TODO)
#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct AppPortConfig {
    #[serde(flatten)]
    port_config: HashMap<String, PortConfig>,
}

impl AppPortConfig {
    pub fn new(port_config_path: impl AsRef<Path> + Display) -> Result<Self, AppError> {
        let content = read_to_string(port_config_path)?;

        Ok(Self {
            port_config: toml::from_str(&content)?,
        })
    }

    pub fn save_port_cfg(&self, port_cfg_path: &str) -> Result<(), AppError> {
        let content = toml::to_string_pretty(&self.port_config)?;
        fs::write(port_cfg_path, content)?;
        Ok(())
    }

    pub fn get_port(&self, port_name: &str) -> Option<&PortConfig> {
        println!("{:?}", self.port_config);
        self.port_config.get(&port_name.to_string())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::port_config::{Color, DataBits, FlowControl, LineEnding, Parity, StopBits};
    use std::path::PathBuf;

    #[test]
    fn test_parse_valid_config() {
        let app_config = AppPortConfig::new("src/config/test/valid_config.toml").unwrap();

        assert_eq!(
            app_config.get_port("port1").unwrap(),
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
        let app_config = AppPortConfig::new("src/config/test/minimal_config.toml").unwrap();

        assert_eq!(
            app_config.get_port("port1").unwrap(),
            &PortConfig {
                path: PathBuf::from("/dev/ttyUSB0"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn test_invalid_baud_fails() {
        assert!(AppPortConfig::new("src/config/test/invalid_baud.toml").is_err());
    }

    #[test]
    fn test_invalid_databits_fails() {
        assert!(AppPortConfig::new("src/config/test/invalid_databits.toml").is_err());
    }

    #[test]
    fn test_invalid_color_fails() {
        assert!(AppPortConfig::new("src/config/test/invalid_color.toml").is_err());
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

    #[test]
    fn test_save_port_config() {}
}
