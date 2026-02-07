//! Serial data logger — writes per-port and combined super log files.

use std::{collections::HashMap, sync::Arc};

use tokio::{
    fs::{self, OpenOptions},
    io::{AsyncWriteExt, BufWriter},
    sync::mpsc,
};

use crate::serial::PortEvent;

/// Runs the logger task, writing incoming serial data to per-port log files
/// and a combined `super.log`.
pub async fn run(mut log_rx: mpsc::UnboundedReceiver<Arc<PortEvent>>) {
    if let Err(e) = fs::create_dir_all("logs").await {
        eprintln!("logger: failed to create logs/ directory: {e}");
        return;
    }

    let super_file = match OpenOptions::new()
        .create(true)
        .append(true)
        .open("logs/super.log")
        .await
    {
        Ok(f) => f,
        Err(e) => {
            eprintln!("logger: failed to open super.log: {e}");
            return;
        }
    };
    let mut super_writer = BufWriter::new(super_file);
    let mut port_writers: HashMap<Arc<str>, BufWriter<fs::File>> = HashMap::new();

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
        if !port_writers.contains_key(port) {
            let filename = format!("logs/{port}.log");
            match OpenOptions::new()
                .create(true)
                .append(true)
                .open(&filename)
                .await
            {
                Ok(f) => {
                    port_writers.insert(port.clone(), BufWriter::new(f));
                }
                Err(e) => {
                    eprintln!("logger: failed to open {filename}: {e}");
                }
            }
        }

        if let Some(w) = port_writers.get_mut(port) {
            let line = format!("[{ts}] {text}\n");
            let _ = w.write_all(line.as_bytes()).await;
            let _ = w.flush().await;
        }

        // Write to super.log
        let super_line = format!("[{ts}] [{port}] {text}\n");
        let _ = super_writer.write_all(super_line.as_bytes()).await;
        let _ = super_writer.flush().await;
    }
}
