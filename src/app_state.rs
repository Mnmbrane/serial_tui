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

    pub fn init(mut self) -> Result<(), AppError> {
        // read config file
        let cfg = fs::read_to_string("config/ports.toml")?;

        self.config.init(cfg.as_str())?;

        Ok(())
    }
}
