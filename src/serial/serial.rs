use core::error;
//
use std::{
    sync::{Arc, RwLock},
    thread::{self, JoinHandle},
    time::Duration,
};

use serialport::SerialPort;

use crate::{
    error::AppError,
    types::{
        port_info::{self, PortInfo},
        port_map::{self, PortMap},
    },
};

pub struct Serial {
    port_map: PortMap,
}

impl Serial {
    pub fn new(port_map: PortMap) -> Self {
        Self { port_map }
    }

    fn spawn_reader(&self, port: Arc<RwLock<PortInfo>>) -> JoinHandle<()> {
        let port = port.clone();
        thread::spawn(move || {
            loop {
                let port_info: PortInfo = port.read().unwrap().clone();
            }
        })
    }

    fn spawn_writer(&self, port: Arc<RwLock<PortInfo>>) -> JoinHandle<()> {
        let port = port.clone();
        thread::spawn(move || loop {})
    }

    /// Iterate through the port map and start each port
    /// This is usually done right after new
    pub fn start_ports(&self) {}

    // Starts reader and writer threads.
    pub fn add_port_and_start(&self) -> Result<&Self, AppError> {
        Ok(self)
    }
}

#[cfg(test)]
mod test {
    use super::*;
}
