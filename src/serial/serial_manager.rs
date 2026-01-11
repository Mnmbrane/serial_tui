use std::{
    collections::HashMap,
    fs::{self, read_to_string},
    hash::Hash,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver, Sender},
    },
    thread::{self, JoinHandle, spawn},
};

use crate::{
    error::AppError,
    serial::{port_connection::PortConnection, port_info::PortInfo},
};

type PortConnectionMap = HashMap<String, Arc<Mutex<PortConnection>>>;
/// Clients can ask to subscribe to it's broadcast
/// Clients can ask to get a reader/writer to a handle
pub struct SerialManager {
    port_conn_map: PortConnectionMap,
    subscribers: Vec<Sender<Vec<u8>>>, // Tune into what the serial ports has to say!
}

// Serial Manager Responsibilities
// 1. Open ports based on config
// 2. Save port mapping back to config (in case editing, adding or deleting)
// 3. Open all writers/readers for serial ports
// 4.
impl SerialManager {
    /// Load port configurations from a TOML file.
    ///
    /// Appends all ports from the file to this map. The TOML file should have
    /// one `[port_name]` section per port.
    pub fn from_file(port_config_path: impl AsRef<Path>) -> Result<Self, AppError> {
        let mut port_conn_map: PortConnectionMap = HashMap::new();

        // Iterate through the ports from the config file
        for (name, port_info) in
            toml::from_str::<HashMap<String, PortInfo>>(read_to_string(port_config_path)?.as_str())?
        {
            // TODO
            //port_conn_map.insert(name, Arc::new(Mutex::new(PortConnection::new(port_info))));
        }

        Ok(Self {
            port_conn_map,
            subscribers: Vec::new(),
        })
    }

    pub fn subscribe_to_broadcast(&mut self) -> Receiver<Vec<u8>> {
        let (tx, rx) = mpsc::channel();
        self.subscribers.push(tx);
        rx
    }

    /// Broadcast to all subsribers
    pub fn broadcast(&mut self, msg: &[u8]) {
        // Only retain Senders
        self.subscribers
            .retain(|sub| sub.send(msg.to_vec()).is_ok());
    }

    // TODO: Remove connection from map
    pub fn remove_connection() {}

    /// Save all port configurations to a TOML file.
    ///
    /// Overwrites the file if it exists. Each port is saved as a separate
    /// `[port_name]` section.
    pub fn save(&mut self, port_cfg_path: impl AsRef<Path>) -> Result<(), AppError> {
        let port_map: HashMap<String, PortInfo> = self
            .port_conn_map
            .iter()
            .map(|(port_name, port_conn)| {
                (
                    port_name.clone(),
                    port_conn.lock().expect("mutex poisoned").info.clone(),
                )
            })
            .collect();

        let content = toml::to_string_pretty(&port_map)?;
        fs::write(port_cfg_path.as_ref(), content)?;
        Ok(())
    }
}
