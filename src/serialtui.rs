use std::{
    sync::{Arc, RwLock},
    thread,
};

use crate::{
    config::{PortConfig, SharedConfig},
    error::AppError,
};

#[derive(Clone)]
pub struct SerialTui {
    config: SharedConfig,
}

const PORT_CONFIG_PATH: &'static str = "config/ports.toml";

impl SerialTui {
    pub fn new() -> Result<Self, AppError> {
        Ok(Self {
            config: SharedConfig::new().from_file(PORT_CONFIG_PATH)?,
        })
    }
}
