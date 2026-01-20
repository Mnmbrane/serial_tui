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

type PortConnectionMap = HashMap<String, Arc<Mutex<PortConnection>>>;

/// Clients can ask to subscribe to it's broadcast
/// Clients can ask to get a reader/writer to a handle
pub struct SerialManager {
    port_conn_map: PortConnectionMap,

    writer_map: HashMap<String, mpsc::Sender<Arc<Vec<u8>>>>,

    /// Broadcast out to clients
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
            port_conn_map: HashMap::new(),
            writer_map: HashMap::new(),
            broadcast: tx,
        }
    }
    /// Load port configurations from a TOML file.
    ///
    /// Appends all ports from the file to this map. The TOML file should have
    /// one `[port_name]` section per port.
    pub fn from_file(&mut self, port_config_path: impl AsRef<Path>) -> Result<(), AppError> {
        // Iterate through the ports from the config file
        for (name, port_info) in
            toml::from_str::<HashMap<String, PortInfo>>(read_to_string(port_config_path)?.as_str())?
        {
            self.open(name, port_info)?;
        }
        Ok(())
    }

    /// Create map with port connections
    /// Open port connection based on port info
    /// Create channels to write to serial ports or read from all serial ports
    pub fn open(&mut self, name: String, port_info: PortInfo) -> Result<(), AppError> {
        // Start each connection
        // spawns the reader and writers
        let mut port_connection = PortConnection::new();

        // Get the writer and insert to map
        self.writer_map.insert(
            name.clone(),
            port_connection.open(port_info, self.broadcast.clone())?,
        );

        // Insert connection into a map
        self.port_conn_map
            .insert(name, Arc::new(Mutex::new(port_connection)));

        Ok(())
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Arc<PortEvent>> {
        self.broadcast.subscribe()
    }

    pub fn send(&self, keys: &[String], data: Vec<u8>) -> Result<(), AppError> {
        let data = Arc::new(data.to_vec());
        for key in keys {
            self.writer_map
                .get(key)
                .ok_or(AppError::InvalidMapKey)?
                .send(data.clone())
                .map_err(|e| AppError::InvalidSend(e))?
        }
        Ok(())
    }

    ///
    /// Overwrites the file if it exists. Each port is saved as a separate
    /// `[port_name]` section.
    pub fn save(&mut self, port_cfg_path: impl AsRef<Path>) -> Result<(), AppError> {
        todo!()
    }
}
