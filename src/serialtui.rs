use crate::{config::AppPortConfig, error::AppError};

#[derive(Clone)]
pub struct SerialTui {
    config: AppPortConfig,
}

const PORT_CONFIG_PATH: &'static str = "config/ports.toml";

impl SerialTui {
    pub fn new() -> Result<Self, AppError> {
        Ok(Self {
            config: AppPortConfig::new(PORT_CONFIG_PATH).unwrap_or_default(),
        })
    }

    fn setup_ports(self) -> Result<Self, AppError> {
        let spawn_ports = |num_ports: u32| {};
        Ok(self)
    }
}
