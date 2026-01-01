use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::Display,
    fs::{self, read_to_string},
    path::Path,
    sync::{Arc, RwLock},
};

use crate::{config::PortConfig, error::AppError};

// Want just 2 differnt configs for now.
// 1. PortConfig - Contains com port details
// 2. MacroConfig - Contains keybindings for VIM Motions (TODO)
#[derive(Serialize, Deserialize, Clone, Default, Debug)]
struct Config {
    #[serde(flatten)]
    cfg_map: HashMap<String, PortConfig>,
}

#[derive(Clone, Default, Debug)]
pub struct SharedConfig {
    cfg_map: HashMap<String, Arc<RwLock<PortConfig>>>,
}

impl From<&Config> for SharedConfig {
    fn from(cfg: &Config) -> Self {
        Self {
            cfg_map: cfg
                .cfg_map
                .clone()
                .into_iter()
                .map(|(k, v)| (k.clone(), Arc::new(RwLock::new(v.clone()))))
                .collect(),
        }
    }
}

impl From<&SharedConfig> for Config {
    fn from(cfg: &SharedConfig) -> Self {
        Self {
            cfg_map: cfg
                .cfg_map
                .clone()
                .into_iter()
                .map(|(k, v)| (k.clone(), v.read().unwrap().clone()))
                .collect(),
        }
    }
}

impl SharedConfig {
    pub fn new() -> Self {
        Self {
            cfg_map: HashMap::new(),
        }
    }

    // append ports from file to the map
    pub fn from_file(mut self, port_config_path: impl AsRef<Path>) -> Result<Self, AppError> {
        let cfg: Config = toml::from_str(read_to_string(port_config_path)?.as_str())?;

        for (name, port) in cfg.cfg_map {
            self.cfg_map.insert(name, Arc::new(RwLock::new(port)));
        }

        Ok(self)
    }

    /// Save port config hash map into a file
    pub fn save(&self, port_cfg_path: impl AsRef<Path>) -> Result<(), AppError> {
        let cfg_map = Config::from(self).cfg_map;
        let content = toml::to_string_pretty(&cfg_map)?;
        fs::write(port_cfg_path.as_ref(), content)?;
        Ok(())
    }

    pub fn insert_port(
        &mut self,
        port_name: impl AsRef<str>,
        port_config: PortConfig,
    ) -> Result<(), AppError> {
        if let Some(_) = self.cfg_map.insert(
            port_name.as_ref().to_string(),
            Arc::new(RwLock::new(port_config)),
        ) {
            Ok(())
        } else {
            Err(AppError::ConfigPortInsert(
                "Unable to insert new port {port_name}",
            ))
        }
    }

    pub fn get_port(&self, port_name: impl AsRef<str>) -> Option<PortConfig> {
        if let Some(port) = self.cfg_map.get(port_name.as_ref()) {
            Some(port.read().unwrap().clone())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::port::{Color, DataBits, FlowControl, LineEnding, Parity, StopBits};
    use std::{env::temp_dir, path::PathBuf};

    #[test]
    fn test_parse_valid_config() {
        let app_config = SharedConfig::new()
            .from_file("src/config/test/valid_config.toml")
            .unwrap();

        assert_eq!(
            app_config.get_port("port1").unwrap(),
            PortConfig {
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
            app_config.get_port("port2").unwrap(),
            PortConfig {
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
        let app_config = SharedConfig::new()
            .from_file("src/config/test/minimal_config.toml")
            .unwrap();

        assert_eq!(
            app_config.get_port("port1").unwrap(),
            PortConfig {
                path: PathBuf::from("/dev/ttyUSB0"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn test_invalid_baud_fails() {
        assert!(
            SharedConfig::new()
                .from_file("src/config/test/invalid_baud.toml")
                .is_err()
        );
    }

    #[test]
    fn test_invalid_databits_fails() {
        assert!(
            SharedConfig::new()
                .from_file("src/config/test/invalid_databits.toml")
                .is_err()
        );
    }

    #[test]
    fn test_invalid_color_fails() {
        assert!(
            SharedConfig::new()
                .from_file("src/config/test/invalid_color.toml")
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

    #[test]
    fn test_save_port_config() {
        let mut app_config = SharedConfig::new();
        let _ = app_config.insert_port(
            "test_port".to_string(),
            PortConfig {
                path: PathBuf::from("/dev/ttyUSB0"),
                baud_rate: 115200,
                data_bits: DataBits::Eight,
                stop_bits: StopBits::One,
                parity: Parity::None,
                flow_control: FlowControl::None,
                line_ending: LineEnding::CRLF,
                color: Color::Green,
            },
        );

        let save_path = temp_dir().as_path().join("save_file_input.toml");
        app_config.save(&save_path).unwrap();

        // Load it back and verify
        let loaded_config = SharedConfig::new().from_file(&save_path).unwrap();

        assert_eq!(
            loaded_config.get_port("test_port").unwrap(),
            app_config.get_port("test_port").unwrap()
        );
        app_config.save(&save_path).unwrap();

        // Clean up
        fs::remove_file(save_path).unwrap();
    }
}
