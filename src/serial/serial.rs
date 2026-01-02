use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    thread,
};

use crate::{config::PortConfig, error::AppError};

pub struct Serial {
    port_cfg_map: HashMap<String, Arc<RwLock<PortConfig>>>,
}

impl Serial {
    pub fn new() -> Self {
        Self {
            port_cfg_map: HashMap::new(),
        }
    }
    pub fn setup_ports(self) -> Result<Self, AppError> {
        let spawn_port = |port: Arc<RwLock<PortConfig>>| {
            let port = port.clone();

            // spawn readers
            thread::spawn(move || {});

            // spawn writers
        };

        Ok(self)
    }

    pub fn insert_port(
        &mut self,
        port_name: impl AsRef<str>,
        port_config: PortConfig,
    ) -> Result<(), AppError> {
        if let Some(_) = self.port_cfg_map.insert(
            port_name.as_ref().to_string(),
            Arc::new(RwLock::new(port_config)),
        ) {
            Ok(())
        } else {
            Err(AppError::ConfigPortInsert(
                "Unable to insert new port {port_name}",
            ))
        }
    }

    pub fn get_port(&self, port_name: impl AsRef<str>) -> Option<PortConfig> {
        if let Some(port) = self.port_cfg_map.get(port_name.as_ref()) {
            Some(port.read().unwrap().clone())
        } else {
            None
        }
    }

    pub fn read_port(self) {}
}

#[cfg(test)]
mod test {
    use super::*;
}
