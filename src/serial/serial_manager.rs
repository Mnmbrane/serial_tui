use std::{
    collections::HashMap,
    fs::{self, read_to_string},
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver, Sender},
    },
    thread::{self, JoinHandle},
};

use crate::{
    error::AppError,
    serial::{PortHandle, port_info::PortInfo},
};

type PortInfoMap = HashMap<String, Arc<Mutex<PortInfo>>>;
type PortChannelMap = HashMap<String, (Sender<Vec<u8>>, Receiver<Vec<u8>>)>;
pub struct SerialManager {
    port_info_map: PortInfoMap,
    subscribers: Vec<Sender<Vec<u8>>>,
}

impl SerialManager {
    /// Load port configurations from a TOML file.
    ///
    /// Appends all ports from the file to this map. The TOML file should have
    /// one `[port_name]` section per port.
    pub fn from_file(port_config_path: impl AsRef<Path>) -> Result<Self, AppError> {
        let mut port_info_map: PortInfoMap = HashMap::new();
        for (name, port_info) in
            toml::from_str::<HashMap<String, PortInfo>>(read_to_string(port_config_path)?.as_str())?
        {
            port_info_map.insert(name, Arc::new(Mutex::new(port_info)));
        }

        Ok(Self {
            port_info_map,
            subscribers: Vec::new(),
        })
    }

    pub fn subscribe_to_broadcast(&mut self) -> Receiver<Vec<u8>> {
        let (tx, rx) = mpsc::channel();
        self.subscribers.push(tx);
        rx
    }

    pub fn broadcast(&mut self, msg: &[u8]) {
        // Only retain Senders
        self.subscribers
            .retain(|sub| sub.send(msg.to_vec()).is_ok());
    }

    /// Save all port configurations to a TOML file.
    ///
    /// Overwrites the file if it exists. Each port is saved as a separate
    /// `[port_name]` section.
    pub fn save(&mut self, port_cfg_path: impl AsRef<Path>) -> Result<(), AppError> {
        let port_map: HashMap<String, PortInfo> = self
            .port_info_map
            .iter()
            .map(|(port_name, port_info)| (port_name.clone(), port_info.lock().unwrap().clone()))
            .collect();

        let content = toml::to_string_pretty(&port_map)?;
        fs::write(port_cfg_path.as_ref(), content)?;
        Ok(())
    }

    /// Spawn reader thread for a particular port name
    fn spawn_reader(&self, path: PathBuf, baud_rate: u32) -> (Receiver<Vec<u8>>, JoinHandle<()>) {
        let (tx, rx) = mpsc::channel();

        (
            rx,
            thread::spawn(move || {
                loop {
                    let mut port_handle = PortHandle::new();
                }
            }),
        )
    }

    fn spawn_writer(&self, path: PathBuf, baud_rate: u32) -> (Sender<Vec<u8>>, JoinHandle<()>) {
        let (tx, rx) = mpsc::channel();
        (
            tx,
            thread::spawn(move || {
                let mut port_handle = PortHandle::new();
                let _ = port_handle.open(path.clone(), baud_rate);

                while let Ok(x) = rx.recv() {
                    port_handle.write(x.as_ref());
                }
            }),
        )
    }
}
