//! Central hub for multiple serial port connections.

use std::{collections::HashMap, fs::read_to_string, path::Path, sync::Arc};

use anyhow::{Context, Result};
use tokio::sync::mpsc;

use crate::config::PortConfig;

use super::{
    SerialError,
    port::{Port, PortEvent},
};

/// Manages multiple serial port connections.
pub struct SerialHub {
    ports: HashMap<Arc<str>, Port>,
    event_tx: mpsc::Sender<Arc<PortEvent>>,
}

impl SerialHub {
    pub fn new() -> (Self, mpsc::Receiver<Arc<PortEvent>>) {
        let (event_tx, event_rx) = mpsc::channel(1024);
        (
            Self {
                ports: HashMap::new(),
                event_tx,
            },
            event_rx,
        )
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
        let name: Arc<str> = name.into();
        let port = Port::open(name.clone(), config, self.event_tx.clone())?;
        self.ports.insert(name, port);
        Ok(())
    }

    pub fn get_config(&self, name: &str) -> Option<&Arc<PortConfig>> {
        self.ports.get(name).map(|p| &p.config)
    }

    pub fn list_ports(&self) -> Vec<(Arc<str>, Arc<PortConfig>)> {
        self.ports
            .iter()
            .map(|(name, port)| (name.clone(), port.config.clone()))
            .collect()
    }

    /// Sends data to one or more ports.
    pub fn send(&self, ports: &[Arc<str>], data: Vec<u8>) -> Result<(), SerialError> {
        for name in ports {
            let port = self
                .ports
                .get(name)
                .ok_or_else(|| SerialError::PortNotFound(name.clone()))?;

            let mut buf = data.clone();
            buf.extend_from_slice(port.config.line_ending.as_bytes());
            //port.writer_tx.send(buf)?;
        }
        Ok(())
    }
}
