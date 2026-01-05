use std::{
    collections::HashMap,
    fs::{self, read_to_string},
    path::Path,
    sync::{Arc, RwLock, mpsc::Sender},
    thread::{self, JoinHandle},
    time::Duration,
};

use serde::{Serialize, Serializer, ser::SerializeMap};
use serialport::SerialPort;

use crate::{error::AppError, serial::port_info::PortInfo};

pub struct Serial {
    /// Config data
    port_map: HashMap<String, Arc<RwLock<PortInfo>>>,

    /// Serial ports to read and write data to
    ser_map: HashMap<String, Arc<RwLock<Box<dyn SerialPort>>>>,

    /// From other components --> writer threads
    writers: HashMap<String, Sender<String>>,
}

impl Serialize for Serial {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.port_map.len()))?;

        for (name, port_config) in &self.port_map {
            // serialize the key
            map.serialize_key(name)?;

            let port_config = port_config.read().expect("Could not serialize!").clone();

            map.serialize_value(&port_config)?;
        }

        map.end()
    }
}

impl Serial {
    /// Load port configurations from a TOML file.
    ///
    /// Appends all ports from the file to this map. The TOML file should have
    /// one `[port_name]` section per port.
    pub fn from_file(port_config_path: impl AsRef<Path>) -> Result<Self, AppError> {
        let mut port_map: HashMap<String, Arc<RwLock<PortInfo>>> = HashMap::new();
        let mut ser_map: HashMap<String, Arc<RwLock<Box<dyn SerialPort>>>> = HashMap::new();
        let mut writers: HashMap<String, Sender<String>> = HashMap::new();

        for (name, port_info) in
            toml::from_str::<HashMap<String, PortInfo>>(read_to_string(port_config_path)?.as_str())?
        {
            port_map.insert(name.clone(), Arc::new(RwLock::new(port_info.clone())));
            ser_map.insert(
                name.clone(),
                Arc::new(RwLock::new(Serial::open(&port_info)?)),
            );
        }

        Ok(Self {
            port_map,
            ser_map,
            writers,
        })
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

    fn spawn_reader(&self, port: Arc<RwLock<PortInfo>>) -> JoinHandle<()> {
        let port = port.clone();
        thread::spawn(move || loop {})
    }

    fn spawn_writer(&self, port: &Arc<RwLock<PortInfo>>) -> JoinHandle<()> {
        let port = port.clone();
        thread::spawn(move || {
            loop {
                todo!("Spawn writers")
            }
        })
    }

    /// Write to 1 or more ports
    fn write(port_list: Vec<&String>) {
        for port in port_list {
            todo!("Write")
        }
    }

    /// Iterate through the port map and start each port
    /// This is usually done right after new
    pub fn start_ports(&self) -> Result<(), AppError> {
        for (name, port_info) in self.port_map.iter() {
            // get path and baud_rate, wait 100ms, open port
            let read = &port_info.read()?;
            let baud_rate = read.baud_rate;
            let path = &read.path;

            let p = serialport::new(path.to_string_lossy(), baud_rate)
                .timeout(Duration::from_millis(100))
                .open()
                .map_err(|e| AppError::SerialOpenError(e.to_string()))?;
        }
        Ok(())
    }

    pub fn open(port_info: &PortInfo) -> Result<Box<dyn SerialPort>, AppError> {
        let path = &port_info.path;
        let baud_rate = port_info.baud_rate;

        serialport::new(path.to_string_lossy(), baud_rate)
            .timeout(Duration::from_millis(100))
            .open()
            .map_err(|e| AppError::PortMapInvalidGet(format!("{e}")))
    }
}
