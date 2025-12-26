use std::{collections::HashMap, fs, path::Path};

use serde::{Serialize, de::DeserializeOwned};

use crate::config::ConfigError;

pub type PortName = String;

pub struct ConfigUtil;

impl ConfigUtil {
    fn read_from_file(path: &str) -> Result<String, ConfigError> {
        let content = fs::read_to_string(path)?;
        Ok(content)
    }

    fn write_to_file(path: &str, contents: &String) -> Result<(), ConfigError> {
        fs::write(path, contents)?;
        Ok(())
    }

    pub fn deserialize_toml<T: DeserializeOwned>(content: &str) -> Result<T, ConfigError> {
        Ok(toml::from_str(content)?)
    }

    pub fn serialize_toml<T: Serialize>(path: &str, contents: &T) -> Result<(), ConfigError> {
        let contents = toml::to_string(contents)?;
        ConfigUtil::write_to_file(path, &contents)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_load() {}
}
