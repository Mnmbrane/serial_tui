//! Central hub for multiple serial port connections.

use std::{
    collections::HashMap,
    fs::read_to_string,
    path::Path,
    sync::{mpsc, Arc, Mutex},
};

use anyhow::{Context, Result};
use tokio::sync::broadcast;

use crate::config::PortConfig;

use super::{
    connection::{Connection, PortEvent},
    SerialError,
};

/// Resources for a single managed port.
struct ManagedPort {
    #[allow(dead_code)]
    connection: Arc<Mutex<Connection>>,
    writer: mpsc::Sender<Arc<Vec<u8>>>,
    config: Arc<PortConfig>,
}

/// Manages multiple serial port connections.
pub struct SerialHub {
    ports: HashMap<String, ManagedPort>,
    broadcast: broadcast::Sender<Arc<PortEvent>>,
}

impl SerialHub {
    /// Creates a new empty hub.
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel::<Arc<PortEvent>>(1024);
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
        let mut connection = Connection::new();
        let name_arc: Arc<str> = name.clone().into();
        let writer = connection.open(name_arc, config.clone(), self.broadcast.clone())?;

        self.ports.insert(
            name,
            ManagedPort {
                connection: Arc::new(Mutex::new(connection)),
                writer,
                config: Arc::new(config),
            },
        );

        Ok(())
    }

    /// Subscribe to all port events.
    pub fn subscribe(&self) -> broadcast::Receiver<Arc<PortEvent>> {
        self.broadcast.subscribe()
    }

    /// Returns all port names.
    #[allow(dead_code)]
    pub fn port_names(&self) -> Vec<String> {
        self.ports.keys().cloned().collect()
    }

    /// Gets the config for a port by name.
    pub fn get_config(&self, name: &str) -> Option<&Arc<PortConfig>> {
        self.ports.get(name).map(|p| &p.config)
    }

    /// Returns all ports as (name, config) pairs.
    pub fn list_ports(&self) -> Vec<(String, Arc<PortConfig>)> {
        self.ports
            .iter()
            .map(|(name, mp)| (name.clone(), mp.config.clone()))
            .collect()
    }

    /// Returns `true` if a port exists.
    #[allow(dead_code)]
    pub fn has_port(&self, name: &str) -> bool {
        self.ports.contains_key(name)
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

            port.writer.send(Arc::new(buf))?;
        }
        Ok(())
    }

    /// Closes and removes a port.
    #[allow(dead_code)]
    pub fn close(&mut self, name: &str) {
        self.ports.remove(name);
    }
}
