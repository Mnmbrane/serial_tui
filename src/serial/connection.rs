//! Port connection management with reader/writer threads.

use std::{
    sync::{mpsc, Arc},
    thread::{self, JoinHandle},
};

use bytes::Bytes;
use tokio::sync::broadcast;

use crate::config::PortConfig;

use super::{handle::Handle, SerialError};

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

/// Manages a single serial port connection with reader/writer threads.
pub struct Connection {
    #[allow(dead_code)]
    writer_thread: Option<JoinHandle<()>>,
    #[allow(dead_code)]
    reader_thread: Option<JoinHandle<()>>,
}

impl Connection {
    pub fn new() -> Self {
        Self {
            writer_thread: None,
            reader_thread: None,
        }
    }

    /// Opens a port and spawns reader/writer threads.
    pub fn open(
        &mut self,
        name: Arc<str>,
        config: PortConfig,
        broadcast_tx: broadcast::Sender<Arc<PortEvent>>,
    ) -> Result<mpsc::Sender<Arc<Vec<u8>>>, SerialError> {
        let (writer_tx, writer_rx) = mpsc::channel();

        let handle = Handle::open(&config.path, config.baud_rate)?;
        let writer_handle = handle.try_clone()?;

        self.writer_thread = Some(Self::spawn_writer(writer_handle, writer_rx));
        self.reader_thread = Some(Self::spawn_reader(name, handle, broadcast_tx));

        Ok(writer_tx)
    }

    fn spawn_reader(
        port_name: Arc<str>,
        mut handle: Handle,
        broadcast: broadcast::Sender<Arc<PortEvent>>,
    ) -> JoinHandle<()> {
        thread::spawn(move || {
            let mut read_buf = [0u8; 1024];
            let mut line_buf = Vec::with_capacity(256);

            loop {
                match handle.read(&mut read_buf) {
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
                    Err(e) => {
                        if !line_buf.is_empty() {
                            let _ = broadcast.send(Arc::new(PortEvent::Data {
                                port: Arc::clone(&port_name),
                                data: Bytes::copy_from_slice(&line_buf),
                            }));
                        }
                        let _ = broadcast.send(Arc::new(PortEvent::Error(e)));
                        break;
                    }
                }
            }
        })
    }

    fn spawn_writer(mut handle: Handle, rx: mpsc::Receiver<Arc<Vec<u8>>>) -> JoinHandle<()> {
        thread::spawn(move || {
            while let Ok(buf) = rx.recv() {
                let _ = handle.write_all(buf.as_ref());
            }
        })
    }
}
