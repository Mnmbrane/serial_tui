pub mod error;
pub mod port_config;

use std::{collections::HashMap, fs, path::PathBuf};

pub use error::ConfigError;
pub use port_config::PortConfig;

pub type PortName = String;

pub struct Config;

impl Config {
    fn load<T: DeserializeOwned(path: &PathBuf) -> Result<String, ConfigError> {
        let content = fs::read_to_string(path)?;
        Ok(content)
    }

    fn save(path: &PathBuf, contents: String) -> Result<(), ConfigError> {
        std::fs::write(path, contents)?;
        Ok(())
    }
}
