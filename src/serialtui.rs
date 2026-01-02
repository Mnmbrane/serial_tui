use crate::{config::PortMap, error::AppError};

#[derive(Clone)]
pub struct SerialTui {
    port_map: PortMap,
}

const PORT_CONFIG_PATH: &'static str = "config/ports.toml";

impl SerialTui {
    pub fn new() -> Result<Self, AppError> {
        Ok(Self {
            port_map: PortMap::new(),
        })
    }
}
