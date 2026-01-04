//! Thread-safe collection of port configurations.

use serde::{Serialize, Serializer, ser::SerializeMap};
use serialport::SerialPort;
use std::{
    collections::{
        HashMap,
        hash_map::{self, Iter, Keys},
    },
    fs::{self, read_to_string},
    iter,
    path::Path,
    ptr::read,
    sync::{Arc, PoisonError, RwLock},
    time::Duration,
};

use crate::{error::AppError, types::port_info::PortInfo};

/// Thread-safe map of named serial port configurations.
///
/// Each port is wrapped in `Arc<RwLock<>>` for safe concurrent access
/// across multiple threads (reader, writer, UI).
#[derive(Clone, Default, Debug)]
pub struct PortMap {
    port_map: HashMap<String, Arc<RwLock<PortInfo>>>,
}

impl PortMap {
    pub fn new() -> Self {
        Self {
            port_map: HashMap::new(),
        }
    }

    /// Load port configurations from a TOML file.
    ///
    /// Appends all ports from the file to this map. The TOML file should have
    /// one `[port_name]` section per port.
    pub fn from_file(mut self, port_config_path: impl AsRef<Path>) -> Result<Self, AppError> {
        for (name, port) in
            toml::from_str::<HashMap<String, PortInfo>>(read_to_string(port_config_path)?.as_str())?
        {
            self.port_map.insert(name, Arc::new(RwLock::new(port)));
        }

        Ok(self)
    }

    /// Save all port configurations to a TOML file.
    ///
    /// Overwrites the file if it exists. Each port is saved as a separate
    /// `[port_name]` section.
    pub fn save(&self, port_cfg_path: impl AsRef<Path>) -> Result<(), AppError> {
        let content = toml::to_string_pretty(self)?;
        fs::write(port_cfg_path.as_ref(), content)?;
        Ok(())
    }

    pub fn insert(&mut self, key: String, port_info: PortInfo) -> Option<Arc<RwLock<PortInfo>>> {
        self.port_map.insert(key, Arc::new(RwLock::new(port_info)))
    }

    pub fn get(&self, name: &str) -> Option<&Arc<RwLock<PortInfo>>> {
        self.port_map.get(name)
    }

    pub fn remove(&mut self, name: &str) -> Option<Arc<RwLock<PortInfo>>> {
        self.port_map.remove(name)
    }

    pub fn contains_key(&self, name: &str) -> bool {
        self.port_map.contains_key(name)
    }

    pub fn len(&self) -> usize {
        self.port_map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.port_map.is_empty()
    }

    pub fn keys(&self) -> Keys<'_, String, Arc<RwLock<PortInfo>>> {
        self.port_map.keys()
    }

    pub fn iter(&self) -> Iter<'_, String, Arc<RwLock<PortInfo>>> {
        self.port_map.iter()
    }

    pub fn open(&self, name: &str) -> Result<Box<dyn SerialPort>, AppError> {
        if let Some(port_info) = self.port_map.get(name) {
            let path = &port_info
                .read()
                .map_err(|e| AppError::PortMapInvalidGet(format!("{e}")))?
                .path;
            let baud_rate = port_info
                .read()
                .map_err(|e| AppError::PortMapInvalidGet(format!("{e}")))?
                .baud_rate;

            serialport::new(path.to_string_lossy(), baud_rate)
                .timeout(Duration::from_millis(100))
                .open()
                .map_err(|e| AppError::PortMapInvalidGet(format!("{e}")))
        } else {
            Err(AppError::PortMapInvalidOpen("Could not open ".into()))
        }
    }
}

impl Serialize for PortMap {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.port_map.len()))?;

        for (name, port_config) in &self.port_map {
            // serialize the key
            map.serialize_key(name)?;

            let port_config = port_config.read().expect("Could not serialize!").clone();

            map.serialize_value(&port_config)?;
        }

        map.end()
    }
}

#[cfg(test)]
mod test {
    use crate::types::{color::Color, port_info::LineEnding};

    use super::*;
    use std::{io::Write, path::PathBuf, str::FromStr};
    use tempfile::{NamedTempFile, tempdir};

    // Helper to create a temp TOML file
    fn create_test_toml(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file.flush().unwrap();
        file
    }

    // Helper to create a test PortConfig
    fn test_port_config() -> PortInfo {
        PortInfo {
            path: PathBuf::from("/dev/ttyUSB0"),
            baud_rate: 115200,
            line_ending: LineEnding::CRLF,
            color: Color::from_str("green").unwrap(),
        }
    }

    // Helper to create a Config with a single port
    fn config_with_port(name: &str, port: PortInfo) -> PortMap {
        let mut config = PortMap::new();
        config
            .port_map
            .insert(name.to_string(), Arc::new(RwLock::new(port)));
        config
    }

    // ==================== from_file tests ====================

    #[test]
    fn from_file_loads_valid_config() {
        let file = create_test_toml(
            r##"
[port1]
path = "/dev/ttyUSB0"
baud_rate = 115200
line_ending = "crlf"
color = "green"

[port2]
path = "/dev/ttyUSB1"
baud_rate = 9600
line_ending = "lf"
color = "#FF8000"
        "##,
        );

        let config = PortMap::new().from_file(file.path()).unwrap();

        let port1 = config.port_map.get("port1").unwrap().read().unwrap();
        let port2 = config.port_map.get("port2").unwrap().read().unwrap();
        assert_eq!(port1.baud_rate, 115200);
        assert_eq!(port2.baud_rate, 9600);
    }

    #[test]
    fn from_file_uses_defaults_for_missing_fields() {
        let file = create_test_toml(
            r##"
[port1]
path = "/dev/ttyUSB0"
        "##,
        );

        let config = PortMap::new().from_file(file.path()).unwrap();

        let port = config.port_map.get("port1").unwrap().read().unwrap();
        assert_eq!(port.path, PathBuf::from("/dev/ttyUSB0"));
        assert_eq!(port.baud_rate, PortInfo::default().baud_rate);
    }

    #[test]
    fn from_file_fails_on_missing_file() {
        let result = PortMap::new().from_file("nonexistent.toml");
        assert!(result.is_err());
    }

    #[test]
    fn from_file_fails_on_invalid_baud() {
        let file = create_test_toml(
            r#"
[port1]
path = "/dev/ttyUSB0"
baud_rate = "not_a_number"
        "#,
        );

        let result = PortMap::new().from_file(file.path());
        assert!(result.is_err());
    }

    #[test]
    fn from_file_fails_on_invalid_color() {
        let file = create_test_toml(
            r#"
[port1]
path = "/dev/ttyUSB0"
color = "invalid_color"
        "#,
        );

        let result = PortMap::new().from_file(file.path());
        assert!(result.is_err());
    }

    // ==================== save tests ====================

    #[test]
    fn save_and_load_roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("roundtrip.toml");

        let config = config_with_port("test_port", test_port_config());
        config.save(&path).unwrap();

        let loaded = PortMap::new().from_file(&path).unwrap();
        let port = loaded.port_map.get("test_port").unwrap().read().unwrap();
        assert_eq!(*port, test_port_config());
    }

    #[test]
    fn save_overwrites_existing_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("overwrite.toml");

        // Save first config
        let config1 = config_with_port("port1", test_port_config());
        config1.save(&path).unwrap();

        // Save different config to same file
        let mut different = test_port_config();
        different.baud_rate = 9600;
        let config2 = config_with_port("port2", different.clone());
        config2.save(&path).unwrap();

        // Load and verify it's the second config
        let loaded = PortMap::new().from_file(&path).unwrap();
        assert!(loaded.port_map.get("port1").is_none());
        let port2 = loaded.port_map.get("port2").unwrap().read().unwrap();
        assert_eq!(*port2, different);
    }

    #[test]
    fn save_preserves_all_fields() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("all_fields.toml");

        let full_port = PortInfo {
            path: PathBuf::from("/dev/ttyACM0"),
            baud_rate: 9600,
            line_ending: LineEnding::LF,
            color: Color(ratatui::style::Color::Rgb(1, 2, 3)),
        };

        let config = config_with_port("full", full_port.clone());
        config.save(&path).unwrap();

        let loaded = PortMap::new().from_file(&path).unwrap();
        let port = loaded.port_map.get("full").unwrap().read().unwrap();
        assert_eq!(*port, full_port);
    }
}
