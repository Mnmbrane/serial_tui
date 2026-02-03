//! Central hub for multiple serial port connections.

use std::{collections::HashMap, fs::read_to_string, path::Path, sync::Arc};

use anyhow::{Context, Result};
use tokio::sync::broadcast;

use crate::config::PortConfig;

use super::{
    SerialError,
    port::{Port, PortEvent},
};

/// Manages multiple serial port connections.
pub struct SerialHub {
    ports: HashMap<String, Port>,
    broadcast: broadcast::Sender<Arc<PortEvent>>,
}

impl SerialHub {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1024);
        Self {
            ports: HashMap::new(),
            broadcast: tx,
        }
    }

    /// Loads and opens all ports from a TOML config file.
    pub fn load_config(&mut self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        let content = read_to_string(path)
            .with_context(|| format!("failed to read config: {}", path.display()))?;

        let ports: HashMap<String, PortConfig> =
            toml::from_str(&content).context("failed to parse config")?;

        for (name, config) in ports {
            if let Err(e) = self.open(name.clone(), config) {
                eprintln!("failed to open port {name}: {e}");
            }
        }
        Ok(())
    }

    /// Opens a serial port and adds it to the hub.
    pub fn open(&mut self, name: String, config: PortConfig) -> Result<(), SerialError> {
        let name_arc: Arc<str> = name.clone().into();
        let port = Port::open(name_arc, config, self.broadcast.clone())?;
        self.ports.insert(name, port);
        Ok(())
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Arc<PortEvent>> {
        self.broadcast.subscribe()
    }

    pub fn get_config(&self, name: &str) -> Option<&Arc<PortConfig>> {
        self.ports.get(name).map(|p| &p.config)
    }

    pub fn list_ports(&self) -> Vec<(String, Arc<PortConfig>)> {
        self.ports
            .iter()
            .map(|(name, port)| (name.clone(), port.config.clone()))
            .collect()
    }

    /// Sends data to one or more ports.
    pub fn send(&self, ports: &[String], data: Vec<u8>) -> Result<(), SerialError> {
        for name in ports {
            let port = self
                .ports
                .get(name)
                .ok_or_else(|| SerialError::PortNotFound(name.clone()))?;

            let mut buf = data.clone();
            buf.extend_from_slice(port.config.line_ending.as_bytes());
            port.writer_tx.send(buf)?;
        }
        Ok(())
    }
}
