pub mod error;
pub mod port_config;
pub use error::ConfigError;
pub use port_config::PortConfig;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs};

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
    pub fn new(port_conf_path: &str) -> Result<Self, ConfigError> {
        // read from toml file
        let content = fs::read_to_string(port_conf_path)?;

        Ok(toml::from_str(content.as_str())?)
    }
}
