pub mod port_config;
pub use port_config::PortConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::AppError;

#[derive(Serialize, Deserialize)]
pub struct PortConfigMap(HashMap<String, PortConfig>);
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

    pub fn init(&mut self, cfg: &str) -> Result<(), AppError> {
        self.port_config = toml::from_str(cfg)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::{path::PathBuf, str::FromStr};

    use super::*;

    #[test]
    fn test_port_cfg() {
        let toml_str = r#"
  [COM1]
  path = "/dev/ttyUSB0"
  baud_rate = 9600

  [COM2]
  path = "/dev/ttyUSB1"
  "#;
        let mut app_config: AppConfig = AppConfig::new();
        app_config.init(toml_str).unwrap();
        assert_eq!(app_config.port_config.len(), 2);
        assert_eq!(app_config.port_config.get("COM1").unwrap().baud_rate, 9600);
        assert_eq!(
            app_config.port_config.get("COM2").unwrap().path,
            PathBuf::from_str("/dev/ttyUSB1").unwrap()
        );
    }
}
