//! Serial data logger — writes per-port and combined super log files.

use std::{
    collections::HashMap,
    fs::{self, File, OpenOptions},
    io::{Seek, Write},
    sync::{Arc, mpsc},
};

use crate::{serial::PortEvent, ui::UiEvent};

/// Events sent to the logger via channel.
pub enum LoggerEvent {
    SerialData(Arc<PortEvent>),
    Purge,
}

/// Serial data logger that writes per-port and combined log files.
pub struct Logger {
    log_rx: mpsc::Receiver<LoggerEvent>,
    ui_tx: mpsc::Sender<UiEvent>,
    super_file: File,
    port_files: HashMap<Arc<str>, File>,
}

impl Logger {
    /// Creates a new logger, setting up the logs directory and super.log file.
    /// Returns `None` if setup fails (notifies the UI).
    pub fn new(
        log_rx: mpsc::Receiver<LoggerEvent>,
        ui_tx: mpsc::Sender<UiEvent>,
    ) -> Option<Self> {
        if let Err(e) = fs::create_dir_all("logs") {
            let _ = ui_tx.send(UiEvent::ShowNotification(
                format!("Logger: failed to create logs/ directory: {e}").into(),
            ));
            return None;
        }

        let super_file = Self::open_log("logs/super.log", &ui_tx)?;

        Some(Self {
            log_rx,
            ui_tx,
            super_file,
            port_files: HashMap::new(),
        })
    }

    /// Runs the logger event loop until the channel closes.
    pub fn run(mut self) {
        while let Ok(event) = self.log_rx.recv() {
            match event {
                LoggerEvent::Purge => self.purge(),
                LoggerEvent::SerialData(data) => self.handle_data(&data),
            }
        }
    }

    fn purge(&mut self) {
        let _ = self.super_file.set_len(0);
        let _ = self.super_file.rewind();
        for file in self.port_files.values_mut() {
            let _ = file.set_len(0);
            let _ = file.rewind();
        }
        let _ = self
            .ui_tx
            .send(UiEvent::ShowNotification("Logs purged.".into()));
    }

    fn handle_data(&mut self, event: &PortEvent) {
        let PortEvent {
            port,
            data,
            timestamp,
        } = event;

        let ts = timestamp.format("%H:%M:%S%.3f");
        let text = String::from_utf8_lossy(data);
        let text = text.trim_end_matches(['\n', '\r']);

        // Write to per-port file
        if let std::collections::hash_map::Entry::Vacant(entry) =
            self.port_files.entry(port.clone())
        {
            if let Some(f) = Self::open_log(&format!("logs/{port}.log"), &self.ui_tx) {
                entry.insert(f);
            }
        }

        if let Some(f) = self.port_files.get_mut(port) {
            let _ = write!(f, "[{ts}] {text}\n");
        }

        // Write to super.log
        let _ = write!(self.super_file, "[{ts}] [{port}] {text}\n");
    }

    fn open_log(path: &str, ui_tx: &mpsc::Sender<UiEvent>) -> Option<File> {
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|e| {
                ui_tx.send(UiEvent::ShowNotification(
                    format!("Logger: failed to open {path}: {e}").into(),
                ))
            })
            .ok()
    }
}
