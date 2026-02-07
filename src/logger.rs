//! Serial data logger — writes per-port and combined super log files.

use std::{collections::HashMap, sync::Arc};

use tokio::{
    fs::{self, File, OpenOptions},
    io::AsyncWriteExt,
    sync::mpsc,
};

use crate::serial::PortEvent;

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
pub async fn run(mut log_rx: mpsc::UnboundedReceiver<Arc<PortEvent>>) {
    if let Err(e) = fs::create_dir_all("logs").await {
        eprintln!("logger: failed to create logs/ directory: {e}");
        return;
    }

    let Some(mut super_file) = open_log("logs/super.log").await else {
        return;
    };
    let mut port_files: HashMap<Arc<str>, File> = HashMap::new();

    while let Some(event) = log_rx.recv().await {
        let PortEvent::Data {
            port,
            data,
            timestamp,
        } = event.as_ref()
        else {
            continue;
        };

        let ts = timestamp.format("%H:%M:%S%.3f");
        let text = String::from_utf8_lossy(data);

        // Write to per-port file
        if let std::collections::hash_map::Entry::Vacant(entry) = port_files.entry(port.clone()) {
            if let Some(f) = open_log(&format!("logs/{port}.log")).await {
                entry.insert(f);
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
