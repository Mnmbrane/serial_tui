use std::{
    collections::HashMap,
    fmt::write,
    fs::{self, read_to_string},
    path::{Path, PathBuf},
    sync::{
        Arc, RwLock,
        mpsc::{self, Receiver, Sender},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use serde::{Deserialize, Serialize, Serializer, ser::SerializeMap};
use serialport::SerialPort;

use crate::{
    error::AppError,
    serial::{PortHandle, port_handle, port_info::PortInfo},
};

type PortInfoMap = HashMap<String, PortInfo>;
type PortChannelMap = HashMap<String, (Sender<Vec<u8>>, Receiver<Vec<u8>>)>;
#[derive(Deserialize)]
pub struct SerialManager {
    port_info_map: PortInfoMap,
    #[serde(skip)]
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
            toml::from_str::<PortInfoMap>(read_to_string(port_config_path)?.as_str())?
        {
            port_info_map.insert(name, port_info);
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

    pub fn broadcast(&mut self, msg: Vec<u8>) {
        self.subscribers.retain(|sub| sub.send(msg.clone()).is_ok());
    }

    /// Save all port configurations to a TOML file.
    ///
    /// Overwrites the file if it exists. Each port is saved as a separate
    /// `[port_name]` section.
    pub fn save(&self, port_cfg_path: impl AsRef<Path>) -> Result<(), AppError> {
        let content = toml::to_string_pretty(self)?;
        fs::write(port_cfg_path.as_ref(), content)?;
        Ok(())
    }

    /// Spawn reader thread for a particular port name
    fn spawn_reader(&self, port_name: &String) -> JoinHandle<()> {
        thread::spawn(move || loop {})
    }

    fn spawn_writer(&self, path: PathBuf, baud_rate: u32) -> (Sender<Vec<u8>>, JoinHandle<()>) {
        let (tx, rx) = mpsc::channel();
        (
            tx,
            thread::spawn(move || {
                let mut port_handle = PortHandle::new();
                let _ = port_handle.open(path.clone(), baud_rate);

                while let Ok(x) = rx.recv() {}
            }),
        )
    }

    /// Iterate through the port map and start each port
    /// This is usually done right after new
    pub fn open_ports(&mut self) -> Result<(), AppError> {
        Ok(())
    }
}

impl Serialize for SerialManager {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.port_info_map.len()))?;

        for (name, port_handle) in &self.port_info_map {
            // serialize the key
            map.serialize_key(name)?;

            map.serialize_value(&port_handle)?;
        }

        map.end()
    }
}
