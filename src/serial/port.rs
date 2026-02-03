//! Single port connection with sync reader/writer threads.

use std::{
    io::Read,
    sync::{mpsc, Arc},
    thread::{self, JoinHandle},
    time::Duration,
};

use bytes::Bytes;
use serialport::SerialPort;
use tokio::sync::broadcast;

use crate::config::PortConfig;

use super::SerialError;

/// Events emitted by serial ports.
pub enum PortEvent {
    Data { port: Arc<str>, data: Bytes },
    Error(SerialError),
}

/// A connected serial port with running reader/writer threads.
pub struct Port {
    pub writer_tx: mpsc::Sender<Vec<u8>>,
    pub config: Arc<PortConfig>,
    #[allow(dead_code)]
    writer_thread: JoinHandle<()>,
    #[allow(dead_code)]
    reader_thread: JoinHandle<()>,
}

impl Port {
    /// Opens a port and spawns reader/writer threads.
    pub fn open(
        name: Arc<str>,
        config: PortConfig,
        event_tx: broadcast::Sender<Arc<PortEvent>>,
    ) -> Result<Self, SerialError> {
        let (writer_tx, writer_rx) = mpsc::channel();

        let port = serialport::new(config.path.to_string_lossy(), config.baud_rate)
            .timeout(Duration::from_millis(10))
            .open()?;
        let writer_port = port.try_clone()?;

        Ok(Self {
            writer_tx,
            config: Arc::new(config),
            writer_thread: spawn_writer(writer_port, writer_rx),
            reader_thread: spawn_reader(name, port, event_tx),
        })
    }
}

fn spawn_reader(
    name: Arc<str>,
    mut port: Box<dyn SerialPort>,
    tx: broadcast::Sender<Arc<PortEvent>>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut buf = [0u8; 1024];
        let mut line = Vec::with_capacity(256);

        loop {
            match port.read(&mut buf) {
                Ok(0) => {}
                Ok(n) => {
                    for &byte in &buf[..n] {
                        if byte == b'\n' || byte == b'\r' {
                            if !line.is_empty() {
                                let _ = tx.send(Arc::new(PortEvent::Data {
                                    port: Arc::clone(&name),
                                    data: Bytes::copy_from_slice(&line),
                                }));
                                line.clear();
                            }
                        } else {
                            line.push(byte);
                        }
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {}
                Err(e) => {
                    if !line.is_empty() {
                        let _ = tx.send(Arc::new(PortEvent::Data {
                            port: Arc::clone(&name),
                            data: Bytes::copy_from_slice(&line),
                        }));
                    }
                    let _ = tx.send(Arc::new(PortEvent::Error(SerialError::Read(e))));
                    break;
                }
            }
        }
    })
}

fn spawn_writer(mut port: Box<dyn SerialPort>, rx: mpsc::Receiver<Vec<u8>>) -> JoinHandle<()> {
    thread::spawn(move || {
        use std::io::Write;
        while let Ok(data) = rx.recv() {
            if port.write_all(&data).is_err() || port.flush().is_err() {
                break;
            }
        }
    })
}
