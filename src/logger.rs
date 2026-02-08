//! Serial data logger — writes per-port and combined super log files.

use std::{collections::HashMap, sync::Arc};

use tokio::{
    fs::{self, File, OpenOptions},
    io::{AsyncSeekExt, AsyncWriteExt},
    sync::mpsc,
};

use crate::serial::PortEvent;

/// Events sent to the logger via channel.
pub enum LoggerEvent {
    SerialData(Arc<PortEvent>),
    Clear,
}

async fn open_log(path: &str) -> Option<File> {
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .await
        .map_err(|e| eprintln!("logger: failed to open {path}: {e}"))
        .ok()
}

/// Runs the logger task, writing incoming serial data to per-port log files
/// and a combined `super.log`.
pub async fn run(mut log_rx: mpsc::UnboundedReceiver<LoggerEvent>) {
    if let Err(e) = fs::create_dir_all("logs").await {
        eprintln!("logger: failed to create logs/ directory: {e}");
        return;
    }

    let Some(mut super_file) = open_log("logs/super.log").await else {
        return;
    };
    let mut port_files: HashMap<Arc<str>, File> = HashMap::new();

    while let Some(event) = log_rx.recv().await {
        match event {
            LoggerEvent::Clear => {
                let _ = super_file.set_len(0).await;
                let _ = super_file.seek(std::io::SeekFrom::Start(0)).await;
                for file in port_files.values_mut() {
                    let _ = file.set_len(0).await;
                    let _ = file.seek(std::io::SeekFrom::Start(0)).await;
                }
            }
            LoggerEvent::SerialData(data) => {
                let PortEvent::Data {
                    port,
                    data,
                    timestamp,
                } = data.as_ref()
                else {
                    continue;
                };

                let ts = timestamp.format("%H:%M:%S%.3f");
                let text = String::from_utf8_lossy(data);
                let text = text.trim_end_matches(['\n', '\r']);

                // Write to per-port file
                if !port_files.contains_key(port) {
                    if let Some(f) = open_log(&format!("logs/{port}.log")).await {
                        port_files.insert(port.clone(), f);
                    }
                }

                if let Some(f) = port_files.get_mut(port) {
                    let _ = f.write_all(format!("[{ts}] {text}\n").as_bytes()).await;
                }

                // Write to super.log
                let _ = super_file
                    .write_all(format!("[{ts}] [{port}] {text}\n").as_bytes())
                    .await;
            }
        }
    }
}
