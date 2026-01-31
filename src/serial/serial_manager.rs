//! Central manager for multiple serial port connections.
//!
//! The `SerialManager` is the main interface between the UI and serial ports.
//! It handles:
//! - Loading port configurations from TOML files
//! - Opening/closing port connections
//! - Broadcasting received data to subscribers
//! - Sending data to one or more ports

use std::{
    collections::HashMap,
    fs::read_to_string,
    path::Path,
    sync::{Arc, Mutex, mpsc},
};

use tokio::sync::broadcast;

use crate::{
    error::AppError,
    serial::{
        port_connection::{PortConnection, PortEvent},
        port_info::PortInfo,
    },
};

/// Internal struct holding all resources for a single managed port.
struct ManagedPort {
    /// The underlying port connection with reader/writer threads
    #[allow(dead_code)] // Kept for graceful shutdown implementation
    connection: Arc<Mutex<PortConnection>>,
    /// Channel sender for writing data to this port
    writer: mpsc::Sender<Arc<Vec<u8>>>,
    /// Shared port configuration
    info: Arc<PortInfo>,
}

/// Manages multiple serial port connections.
///
/// Provides a pub/sub interface for receiving data from all ports via
/// a broadcast channel. Clients can subscribe with `subscribe()` and
/// send data to specific ports with `send()`.
pub struct SerialManager {
    /// Map of port name -> managed port resources
    ports: HashMap<String, ManagedPort>,
    /// Broadcast channel for all port events (shared sender)
    broadcast: broadcast::Sender<Arc<PortEvent>>,
}

impl SerialManager {
    /// Creates a new empty serial manager.
    ///
    /// Initializes the broadcast channel with capacity for 1024 events.
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel::<Arc<PortEvent>>(1024);
        Self {
            ports: HashMap::new(),
            broadcast: tx,
        }
    }

    /// Loads and opens all ports from a TOML configuration file.
    ///
    /// The TOML file should have one `[port_name]` section per port:
    /// ```toml
    /// [my_device]
    /// path = "/dev/ttyUSB0"
    /// baud_rate = 115200
    /// ```
    pub fn load_config(&mut self, port_config_path: impl AsRef<Path>) -> Result<(), AppError> {
        for (name, port_info) in
            toml::from_str::<HashMap<String, PortInfo>>(read_to_string(port_config_path)?.as_str())?
        {
            self.open(name, port_info)?;
        }
        Ok(())
    }

    /// Opens a serial port and adds it to the manager.
    ///
    /// Spawns reader/writer threads for the port. The port will start
    /// broadcasting received data immediately.
    pub fn open(&mut self, name: String, port_info: PortInfo) -> Result<(), AppError> {
        let mut connection = PortConnection::new();
        // Convert to Arc<str> for cheap cloning in the reader thread
        let name_arc: Arc<str> = name.clone().into();
        let writer = connection.open(name_arc, port_info.clone(), self.broadcast.clone())?;

        self.ports.insert(
            name,
            ManagedPort {
                connection: Arc::new(Mutex::new(connection)),
                writer,
                info: Arc::new(port_info),
            },
        );

        Ok(())
    }

    /// Creates a new subscriber to receive all port events.
    ///
    /// Returns a receiver that will get `PortEvent::Data`, `PortEvent::Error`, etc.
    /// from all managed ports.
    pub fn subscribe(&self) -> broadcast::Receiver<Arc<PortEvent>> {
        self.broadcast.subscribe()
    }

    /// Returns a list of all port names.
    #[allow(dead_code)]
    pub fn get_port_names(&self) -> Vec<String> {
        self.ports.keys().cloned().collect()
    }

    /// Gets the configuration for a port by name.
    pub fn get_port_info(&self, name: &str) -> Option<&Arc<PortInfo>> {
        self.ports.get(name).map(|p| &p.info)
    }

    /// Returns all ports as (name, info) pairs.
    ///
    /// Useful for UI display. The `Arc<PortInfo>` allows cheap cloning.
    pub fn get_port_list(&self) -> Vec<(String, Arc<PortInfo>)> {
        self.ports
            .iter()
            .map(|(name, mp)| (name.clone(), mp.info.clone()))
            .collect()
    }

    /// Returns `true` if a port with the given name exists.
    #[allow(dead_code)]
    pub fn has_port(&self, name: &str) -> bool {
        self.ports.contains_key(name)
    }

    /// Sends data to one or more ports.
    ///
    /// Appends the configured line ending for each port. Since ports may have
    /// different line endings, data is built per-port.
    pub fn send(&self, keys: &[String], data: Vec<u8>) -> Result<(), AppError> {
        for key in keys {
            let port = self.ports.get(key).ok_or(AppError::InvalidMapKey)?;

            // Build data with this port's line ending
            let mut buf = data.clone();
            buf.extend_from_slice(port.info.line_ending.as_bytes());

            port.writer
                .send(Arc::new(buf))
                .map_err(AppError::InvalidSend)?;
        }
        Ok(())
    }

    /// Closes and removes a port from the manager.
    ///
    /// The port's reader/writer threads will terminate.
    #[allow(dead_code)]
    pub fn close(&mut self, name: &str) -> Result<(), AppError> {
        self.ports.remove(name);
        Ok(())
    }

    /// Saves all port configurations to a TOML file.
    ///
    /// Overwrites the file if it exists. Each port is saved as a separate
    /// `[port_name]` section.
    #[allow(dead_code)]
    pub fn save(&mut self, _port_cfg_path: impl AsRef<Path>) -> Result<(), AppError> {
        todo!()
    }
}
