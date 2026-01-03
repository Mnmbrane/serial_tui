//! SerialTui responsibilities
//! 1. Configure using toml files
//! 2. Spawn serial reader and writers
//! 3. Start logging thread
//! 4. Start UI(Ratatui)
use std::{
    sync::{Arc, RwLock},
    thread,
};

use crate::{error::AppError, types::port_map::PortMap};

pub struct Serial {
    port_map: PortMap,
}

impl Serial {
    pub fn new(port_map: PortMap) -> Self {
        Self { port_map }
    }

    /// Spawn ports an
    pub fn start_(self) -> Result<Self, AppError> {
        let spawn_port = |port: Arc<RwLock<PortMap>>| {
            let port = port.clone();

            // spawn readers
            thread::spawn(move || {});

            // spawn writers
        };

        Ok(self)
    }
}

#[cfg(test)]
mod test {
    use super::*;
}
