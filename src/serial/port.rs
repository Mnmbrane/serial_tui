//! Single port connection with async reader/writer tasks.

use std::sync::Arc;

use bytes::Bytes;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::mpsc,
};
use tokio_serial::SerialPortBuilderExt;

use crate::config::PortConfig;

use super::SerialError;

/// Events emitted by serial ports.
pub enum PortEvent {
    Data { port: Arc<str>, data: Bytes },
    Error(SerialError),
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
        event_tx: mpsc::Sender<Arc<PortEvent>>,
    ) -> Result<Self, SerialError> {
        let port = tokio_serial::new(config.path.to_string_lossy(), config.baud_rate)
            .open_native_async()?;

        let (mut reader, mut writer) = tokio::io::split(port);

        // Spawn reader task
        let reader_name = name.clone();
        let reader_tx = event_tx.clone();
        tokio::spawn(async move {
            let mut buf = [0u8; 1024];
            loop {
                match reader.read(&mut buf).await {
                    Ok(0) => break,
                    Ok(n) => {
                        let data = Bytes::copy_from_slice(&buf[..n]);
                        let event = Arc::new(PortEvent::Data {
                            port: reader_name.clone(),
                            data,
                        });
                        if reader_tx.send(event).await.is_err() {
                            break; // receiver dropped
                        }
                    }
                    Err(e) => {
                        let _ = reader_tx
                            .send(Arc::new(PortEvent::Error(SerialError::Read(e))))
                            .await;
                        break;
                    }
                }
            }
        });

        // Spawn writer task
        let (writer_tx, mut writer_rx) = mpsc::channel::<Bytes>(32);
        tokio::spawn(async move {
            while let Some(data) = writer_rx.recv().await {
                if writer.write_all(&data).await.is_err() {
                    break;
                }
            }
        });

        let config = Arc::new(config);
        Ok(Port { writer_tx, config })
    }
}
