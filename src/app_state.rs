use crate::config::{AppConfig, ConfigError};
pub struct AppState {
    config: AppConfig,
}

impl AppState {
    pub fn new(port_conf_path: &str) -> Result<Self, ConfigError> {
        Ok(Self {
            config: AppConfig::new(port_conf_path)?,
        })
    }
}
