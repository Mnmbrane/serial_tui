use std::fs;

use crate::{config::AppConfig, error::AppError};
pub struct AppState {
    config: AppConfig,
}

impl AppState {
    pub fn new() -> Result<Self, AppError> {
        Ok(Self {
            config: AppConfig::new(),
        })
    }

    pub fn configure(mut self, port_cfg_path: &str) -> Result<(), AppError> {
        // read config file for ports
        let cfg = fs::read_to_string(port_cfg_path)?;

        // TODO: Macro config

        self.config.init(cfg.as_str())?;

        Ok(())
    }
}
