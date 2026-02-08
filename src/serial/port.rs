//! Single port connection with async reader/writer tasks.

use std::sync::Arc;

use bytes::Bytes;
use chrono::{DateTime, Local};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::mpsc,
};
use tokio_serial::SerialPortBuilderExt;

use crate::{config::PortConfig, logger::LoggerEvent, serial::SerialError, ui::UiEvent};

/// Events emitted by serial ports.
pub enum PortEvent {
    Data {
        port: Arc<str>,
        data: Bytes,
        timestamp: DateTime<Local>,
    },
}

/// A connected serial port with running reader/writer tasks.
pub struct Port {
    pub writer_tx: mpsc::Sender<Bytes>,
    pub config: Arc<PortConfig>,
}

impl Port {
    /// Opens a port and spawns reader/writer tasks.
    pub fn open(
        name: Arc<str>,
        config: PortConfig,
        ui_tx: mpsc::UnboundedSender<UiEvent>,
        log_tx: mpsc::UnboundedSender<LoggerEvent>,
    ) -> Result<Self, SerialError> {
        let port = tokio_serial::new(config.path.to_string_lossy(), config.baud_rate)
            .open_native_async()?;

        let (mut reader, mut writer) = tokio::io::split(port);

        // Spawn reader task
        let reader_name = name.clone();
        let reader_ui_tx = ui_tx.clone();
        tokio::spawn(async move {
            let mut buf = [0u8; 1024];
            loop {
                match reader.read(&mut buf).await {
                    Ok(0) => break,
                    Ok(n) => {
                        let data = Bytes::copy_from_slice(&buf[..n]);
                        let port_event = PortEvent::Data {
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
                    Err(e) => {
                        let _ = reader_ui_tx.send(UiEvent::ShowNotification(
                            format!("{reader_name}: read error: {e}").into(),
                        ));
                        break;
                    }
                }
            }
        });

        // Spawn writer task
        let (writer_tx, mut writer_rx) = mpsc::channel::<Bytes>(32);
        let writer_name = name;
        tokio::spawn(async move {
            while let Some(data) = writer_rx.recv().await {
                if let Err(e) = writer.write_all(&data).await {
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
