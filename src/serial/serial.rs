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
    pub fn setup_ports(self) -> Result<Self, AppError> {
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
