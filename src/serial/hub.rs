//! Central hub for multiple serial port connections.

use std::{collections::HashMap, fs::read_to_string, path::Path, sync::Arc};

use anyhow::{Context, Result};
use bytes::Bytes;
use tokio::sync::mpsc;

use crate::{config::PortConfig, notify::Notify};

use super::{
    SerialError,
    port::{Port, PortEvent},
};

/// Manages multiple serial port connections.
pub struct SerialHub {
    ports: HashMap<Arc<str>, Port>,
    port_recv_chan_tx: mpsc::UnboundedSender<Arc<PortEvent>>,
    log_tx: mpsc::UnboundedSender<Arc<PortEvent>>,
    notify_tx: mpsc::UnboundedSender<Notify>,
}

impl SerialHub {
    /// Creates a new hub and returns the event receiver for the port data
    pub fn new(
        notify_tx: mpsc::UnboundedSender<Notify>,
        log_tx: mpsc::UnboundedSender<Arc<PortEvent>>,
    ) -> (Self, mpsc::UnboundedReceiver<Arc<PortEvent>>) {
        let (port_recv_chan_tx, port_recv_chan_rx) = mpsc::unbounded_channel();
        (
            Self {
                ports: HashMap::new(),
                port_recv_chan_tx,
                log_tx,
                notify_tx,
            },
            port_recv_chan_rx,
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
        let port = Port::open(
            name.clone(),
            config,
            self.port_recv_chan_tx.clone(),
            self.log_tx.clone(),
            self.notify_tx.clone(),
        )?;
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
    pub fn send(&self, ports: &[Arc<str>], data: Bytes) -> Result<(), SerialError> {
        for name in ports {
            let port = self
                .ports
                .get(name)
                .ok_or_else(|| SerialError::PortNotFound(name.clone()))?;

            port.writer_tx.try_send(data.clone())?;
        }
        Ok(())
    }
}
