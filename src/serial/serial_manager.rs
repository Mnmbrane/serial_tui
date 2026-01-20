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

/// Holds connection and writer for a single port
struct ManagedPort {
    connection: Arc<Mutex<PortConnection>>,
    writer: mpsc::Sender<Arc<Vec<u8>>>,
    info: PortInfo,
}

/// Clients can ask to subscribe to its broadcast
/// Clients can ask to get a reader/writer to a handle
pub struct SerialManager {
    ports: HashMap<String, ManagedPort>,
    broadcast: broadcast::Sender<Arc<PortEvent>>,
}

// Serial Manager Responsibilities
// 1. Open ports based on config
// 2. Save port mapping back to config (in case editing, adding or deleting)
// 3. Open all writers/readers for serial ports
impl SerialManager {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel::<Arc<PortEvent>>(1024);
        Self {
            ports: HashMap::new(),
            broadcast: tx,
        }
    }

    /// Load port configurations from a TOML file.
    ///
    /// Appends all ports from the file to this map. The TOML file should have
    /// one `[port_name]` section per port.
    pub fn from_file(&mut self, port_config_path: impl AsRef<Path>) -> Result<(), AppError> {
        for (name, port_info) in
            toml::from_str::<HashMap<String, PortInfo>>(read_to_string(port_config_path)?.as_str())?
        {
            self.open(name, port_info)?;
        }
        Ok(())
    }

    /// Open a port connection and store it
    pub fn open(&mut self, name: String, port_info: PortInfo) -> Result<(), AppError> {
        let mut connection = PortConnection::new();
        let writer = connection.open(port_info.clone(), self.broadcast.clone())?;

        self.ports.insert(
            name,
            ManagedPort {
                connection: Arc::new(Mutex::new(connection)),
                writer,
                info: port_info,
            },
        );

        Ok(())
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Arc<PortEvent>> {
        self.broadcast.subscribe()
    }

    /// Get list of port names
    pub fn get_port_names(&self) -> Vec<String> {
        self.ports.keys().cloned().collect()
    }

    /// Get port info by name
    pub fn get_port_info(&self, name: &str) -> Option<&PortInfo> {
        self.ports.get(name).map(|p| &p.info)
    }

    /// Check if port exists
    pub fn has_port(&self, name: &str) -> bool {
        self.ports.contains_key(name)
    }

    /// Send data to specified ports
    pub fn send(&self, keys: &[String], data: Vec<u8>) -> Result<(), AppError> {
        let data = Arc::new(data);
        for key in keys {
            self.ports
                .get(key)
                .ok_or(AppError::InvalidMapKey)?
                .writer
                .send(data.clone())
                .map_err(|e| AppError::InvalidSend(e))?;
        }
        Ok(())
    }

    /// Close a port by name
    pub fn close(&mut self, name: &str) -> Result<(), AppError> {
        self.ports.remove(name);
        Ok(())
    }

    /// Overwrites the file if it exists. Each port is saved as a separate
    /// `[port_name]` section.
    pub fn save(&mut self, port_cfg_path: impl AsRef<Path>) -> Result<(), AppError> {
        todo!()
    }
}
