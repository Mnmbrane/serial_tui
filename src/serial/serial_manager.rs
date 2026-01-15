use std::{
    collections::HashMap,
    fs::read_to_string,
    path::Path,
    sync::{Arc, Mutex, mpsc::Sender},
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
    writer_tx: Option<Sender<PortEvent>>,
    broadcast: Option<broadcast::Sender<PortEvent>>,
}

// Serial Manager Responsibilities
// 1. Open ports based on config
// 2. Save port mapping back to config (in case editing, adding or deleting)
// 3. Open all writers/readers for serial ports
impl SerialManager {
    pub fn new() -> Self {
        Self {
            port_conn_map: HashMap::new(),
            writer_tx: None,
            broadcast: None,
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
            self.open_connection(name, port_info)?;
        }
        Ok(())
    }

    /// Create map with port connections
    /// Open port connection based on port info
    /// Create channels to write to serial ports or read from all serial ports
    pub fn open_connection(&mut self, name: String, port_info: PortInfo) -> Result<(), AppError> {
        // Start each connection
        // spawns the reader and writers
        let mut port_connection = PortConnection::new();

        let (tx, rx) = port_connection.open(port_info)?;
        self.writer_tx = Some(tx);

        // Insert connection into a map
        self.port_conn_map
            .insert(name, Arc::new(Mutex::new(port_connection)));
        Ok(())
    }

    // TODO: Remove connection from map
    pub fn remove_connection() {}

    /// Save all port configurations to a TOML file.
    ///
    /// Overwrites the file if it exists. Each port is saved as a separate
    /// `[port_name]` section.
    pub fn save(&mut self, port_cfg_path: impl AsRef<Path>) -> Result<(), AppError> {
        todo!()
    }
}
