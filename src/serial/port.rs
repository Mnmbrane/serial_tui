//! Single port connection with reader/writer threads.

use std::{
    io::{Read, Write},
    sync::{Arc, mpsc},
    time::Duration,
};

use bytes::Bytes;
use chrono::{DateTime, Local};

use crate::{config::PortConfig, logger::LoggerEvent, serial::SerialError, ui::UiEvent};

/// Event emitted when a serial port receives data.
pub struct PortEvent {
    pub port: Arc<str>,
    pub data: Bytes,
    pub timestamp: DateTime<Local>,
}

/// A connected serial port with running reader/writer threads.
pub struct Port {
    pub writer_tx: mpsc::SyncSender<Bytes>,
    pub config: Arc<PortConfig>,
}

impl Port {
    /// Opens a port and spawns reader/writer threads.
    pub fn open(
        name: Arc<str>,
        config: PortConfig,
        ui_tx: mpsc::Sender<UiEvent>,
        log_tx: mpsc::Sender<LoggerEvent>,
    ) -> Result<Self, SerialError> {
        let mut port = serialport::new(config.path.to_string_lossy(), config.baud_rate)
            .timeout(Duration::from_millis(10))
            .open_native()?;

        let mut writer_port = port.try_clone_native()?;

        // Spawn reader thread
        let reader_name = name.clone();
        let reader_ui_tx = ui_tx.clone();
        std::thread::spawn(move || {
            let mut buf = [0u8; 1024];
            loop {
                match port.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        let data = Bytes::copy_from_slice(&buf[..n]);
                        let port_event = PortEvent {
                            port: reader_name.clone(),
                            data,
                            timestamp: Local::now(),
                        };

                        let event = Arc::new(port_event);
                        let _ = log_tx.send(LoggerEvent::SerialData(event.clone()));

                        if reader_ui_tx.send(UiEvent::PortData(event)).is_err() {
                            break; // receiver dropped
                        }
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::TimedOut => continue,
                    Err(e) => {
                        let _ = reader_ui_tx.send(UiEvent::ShowNotification(
                            format!("{reader_name}: read error: {e}").into(),
                        ));
                        break;
                    }
                }
            }
        });

        // Spawn writer thread
        let (writer_tx, writer_rx) = mpsc::sync_channel::<Bytes>(32);
        let writer_name = name;
        std::thread::spawn(move || {
            while let Ok(data) = writer_rx.recv() {
                if let Err(e) = writer_port.write_all(&data) {
                    let _ = ui_tx.send(UiEvent::ShowNotification(
                        format!("{writer_name}: write error: {e}").into(),
                    ));
                    break;
                }
            }
        });

        let config = Arc::new(config);
        Ok(Port { writer_tx, config })
    }
}
