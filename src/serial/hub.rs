//! Central hub for multiple serial port connections.

use std::{collections::HashMap, fs::read_to_string, path::Path, sync::Arc};

use anyhow::{Context, Result};
use bytes::Bytes;
use tokio::sync::mpsc;

/// Commands sent to the hub via channel.
pub enum HubEvent {
    /// Send data to various ports
    Send { ports: Vec<Arc<str>>, data: Bytes },
    /// Send line ending to ports
    SendLineEnding { ports: Vec<Arc<str>> },
}

use crate::{config::PortConfig, logger::LoggerEvent, ui::UiEvent};

use super::{SerialError, port::Port};

/// Manages multiple serial port connections.
pub struct SerialHub {
    ports: HashMap<Arc<str>, Port>,
    ui_tx: mpsc::UnboundedSender<UiEvent>,
    log_tx: mpsc::UnboundedSender<LoggerEvent>,
}

impl SerialHub {
    /// Creates a new hub and returns the event receiver for the port data
    pub fn new(
        ui_tx: mpsc::UnboundedSender<UiEvent>,
        log_tx: mpsc::UnboundedSender<LoggerEvent>,
    ) -> Self {
        Self {
            ports: HashMap::new(),
            ui_tx,
            log_tx,
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
        let name: Arc<str> = name.into();
        let port = Port::open(
            name.clone(),
            config,
            self.ui_tx.clone(),
            self.log_tx.clone(),
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

    /// Sends each port's configured line ending.
    pub fn send_line_ending(&self, ports: &[Arc<str>]) -> Result<(), SerialError> {
        for name in ports {
            let port = self
                .ports
                .get(name)
                .ok_or_else(|| SerialError::PortNotFound(name.clone()))?;

            let ending = port.config.line_ending.as_bytes();
            port.writer_tx.try_send(Bytes::from_static(ending))?;
        }
        Ok(())
    }
}
