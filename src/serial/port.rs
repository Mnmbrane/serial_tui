//! Single port connection with sync reader/writer threads.

use std::{
    path::Path,
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
    /// Data received from a port.
    Data { port: Arc<str>, data: Bytes },
    /// Error occurred on a port.
    Error(SerialError),
    #[allow(dead_code)]
    Disconnected(String),
    #[allow(dead_code)]
    PortAdded(String),
    #[allow(dead_code)]
    PortRemoved(String),
}

/// A connected serial port with running reader/writer threads.
pub struct Port {
    pub writer_tx: mpsc::Sender<Arc<Vec<u8>>>,
    pub config: Arc<PortConfig>,
    #[allow(dead_code)]
    writer_thread: Option<JoinHandle<()>>,
    #[allow(dead_code)]
    reader_thread: Option<JoinHandle<()>>,
}

impl Port {
    /// Opens a port and spawns reader/writer threads.
    pub fn open(
        name: Arc<str>,
        config: PortConfig,
        broadcast_tx: broadcast::Sender<Arc<PortEvent>>,
    ) -> Result<Self, SerialError> {
        let (writer_tx, writer_rx) = mpsc::channel();

        let port = open_serial_port(&config.path, config.baud_rate)?;
        let writer_port = port.try_clone()?;

        let writer_thread = Some(spawn_writer(writer_port, writer_rx));
        let reader_thread = Some(spawn_reader(name, port, broadcast_tx));

        Ok(Self {
            writer_tx,
            config: Arc::new(config),
            writer_thread,
            reader_thread,
        })
    }
}

/// Opens a serial port at the given path with specified baud rate.
fn open_serial_port(path: &Path, baud_rate: u32) -> Result<Box<dyn SerialPort>, SerialError> {
    let port = serialport::new(path.to_string_lossy(), baud_rate)
        .timeout(Duration::from_millis(10))
        .open()?;
    Ok(port)
}

fn spawn_reader(
    port_name: Arc<str>,
    mut port: Box<dyn SerialPort>,
    broadcast: broadcast::Sender<Arc<PortEvent>>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut read_buf = [0u8; 1024];
        let mut line_buf = Vec::with_capacity(256);

        loop {
            match port.read(&mut read_buf) {
                Ok(0) => continue,
                Ok(n) => {
                    for &byte in &read_buf[..n] {
                        if byte == b'\n' || byte == b'\r' {
                            if !line_buf.is_empty() {
                                let _ = broadcast.send(Arc::new(PortEvent::Data {
                                    port: Arc::clone(&port_name),
                                    data: Bytes::copy_from_slice(&line_buf),
                                }));
                                line_buf.clear();
                            }
                        } else {
                            line_buf.push(byte);
                        }
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::TimedOut => continue,
                Err(e) => {
                    if !line_buf.is_empty() {
                        let _ = broadcast.send(Arc::new(PortEvent::Data {
                            port: Arc::clone(&port_name),
                            data: Bytes::copy_from_slice(&line_buf),
                        }));
                    }
                    let _ = broadcast.send(Arc::new(PortEvent::Error(SerialError::Read(e))));
                    break;
                }
            }
        }
    })
}

fn spawn_writer(mut port: Box<dyn SerialPort>, rx: mpsc::Receiver<Arc<Vec<u8>>>) -> JoinHandle<()> {
    thread::spawn(move || {
        while let Ok(buf) = rx.recv() {
            if let Err(e) = port.write_all(buf.as_ref()) {
                eprintln!("write error: {e}");
                break;
            }
            if let Err(e) = port.flush() {
                eprintln!("flush error: {e}");
                break;
            }
        }
    })
}
