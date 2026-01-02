use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
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
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::port::{Color, LineEnding};
    use serialport::{DataBits, FlowControl, Parity, StopBits};
    use std::path::PathBuf;
    use tempfile::tempdir;

    // Helper to create a test config
    fn test_port_config() -> PortConfig {
        PortConfig {
            path: PathBuf::from("/dev/ttyUSB0"),
            baud_rate: 115200,
            data_bits: DataBits::Eight,
            stop_bits: StopBits::One,
            parity: Parity::None,
            flow_control: FlowControl::None,
            line_ending: LineEnding::CRLF,
            color: Color::Named("Green".into()),
        }
    }

    // ==================== from_file tests ====================

    #[test]
    fn from_file_loads_valid_config() {
        let config = SharedConfig::new()
            .from_file("src/config/test/valid_config.toml")
            .unwrap();

        let port1 = config.cfg_map.get("port1").unwrap().read().unwrap();
        let port2 = config.cfg_map.get("port2").unwrap().read().unwrap();
        assert_eq!(port1.baud_rate, 115200);
        assert_eq!(port2.baud_rate, 9600);
    }

    #[test]
    fn from_file_uses_defaults_for_missing_fields() {
        let config = SharedConfig::new()
            .from_file("src/config/test/minimal_config.toml")
            .unwrap();

        let port = config.cfg_map.get("port1").unwrap().read().unwrap();
        assert_eq!(port.path, PathBuf::from("/dev/ttyUSB0"));
        assert_eq!(port.baud_rate, PortConfig::default().baud_rate);
        assert_eq!(port.data_bits, PortConfig::default().data_bits);
    }

    #[test]
    fn from_file_fails_on_missing_file() {
        let result = SharedConfig::new().from_file("nonexistent.toml");
        assert!(result.is_err());
    }

    #[test]
    fn from_file_fails_on_invalid_baud() {
        let result = SharedConfig::new().from_file("src/config/test/invalid_baud.toml");
        assert!(result.is_err());
    }

    #[test]
    fn from_file_fails_on_invalid_databits() {
        let result = SharedConfig::new().from_file("src/config/test/invalid_databits.toml");
        assert!(result.is_err());
    }

    #[test]
    fn from_file_fails_on_invalid_color() {
        let result = SharedConfig::new().from_file("src/config/test/invalid_color.toml");
        assert!(result.is_err());
    }

    // ==================== save tests ====================

    #[test]
    fn save_and_load_roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("roundtrip.toml");

        // Create SharedConfig from a raw Config
        let mut raw = Config {
            cfg_map: HashMap::new(),
        };
        raw.cfg_map
            .insert("test_port".to_string(), test_port_config());
        let config = SharedConfig::from(&raw);

        config.save(&path).unwrap();

        let loaded = SharedConfig::new().from_file(&path).unwrap();
        let port = loaded.cfg_map.get("test_port").unwrap().read().unwrap();
        assert_eq!(*port, test_port_config());
    }

    #[test]
    fn save_overwrites_existing_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("overwrite.toml");

        // Save first config
        let mut raw1 = Config {
            cfg_map: HashMap::new(),
        };
        raw1.cfg_map.insert("port1".to_string(), test_port_config());
        let config1 = SharedConfig::from(&raw1);
        config1.save(&path).unwrap();

        // Save different config to same file
        let mut different = test_port_config();
        different.baud_rate = 9600;
        let mut raw2 = Config {
            cfg_map: HashMap::new(),
        };
        raw2.cfg_map.insert("port2".to_string(), different.clone());
        let config2 = SharedConfig::from(&raw2);
        config2.save(&path).unwrap();

        // Load and verify it's the second config
        let loaded = SharedConfig::new().from_file(&path).unwrap();
        assert!(loaded.cfg_map.get("port1").is_none());
        let port2 = loaded.cfg_map.get("port2").unwrap().read().unwrap();
        assert_eq!(*port2, different);
    }

    #[test]
    fn save_preserves_all_fields() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("all_fields.toml");

        let full_config = PortConfig {
            path: PathBuf::from("/dev/ttyACM0"),
            baud_rate: 9600,
            data_bits: DataBits::Seven,
            stop_bits: StopBits::Two,
            parity: Parity::Even,
            flow_control: FlowControl::Hardware,
            line_ending: LineEnding::LF,
            color: Color::Rgb(255, 128, 0),
        };

        let mut raw = Config {
            cfg_map: HashMap::new(),
        };
        raw.cfg_map.insert("full".to_string(), full_config.clone());
        let config = SharedConfig::from(&raw);
        config.save(&path).unwrap();

        let loaded = SharedConfig::new().from_file(&path).unwrap();
        let port = loaded.cfg_map.get("full").unwrap().read().unwrap();
        assert_eq!(*port, full_config);
    }
}
