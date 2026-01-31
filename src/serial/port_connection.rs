//! Port connection management with reader/writer threads.

use std::{
    sync::{Arc, mpsc},
    thread::{self, JoinHandle},
};

use bytes::Bytes;
use tokio::sync::broadcast;

use super::{SerialError, port_handle::PortHandle, port_info::PortInfo};

/// Events emitted by serial ports.
pub enum PortEvent {
    /// Data received from a port. Uses Arc<str> and Bytes for cheap cloning.
    Data { port: Arc<str>, data: Bytes },
    /// Error occurred on a port.
    Error(SerialError),
    #[allow(dead_code)] // Will be used when implementing port hot-plug
    Disconnected(String),
    #[allow(dead_code)]
    PortAdded(String),
    #[allow(dead_code)]
    PortRemoved(String),
}

/// Manages a single serial port connection with reader/writer threads.
pub struct PortConnection {
    /// Thread handle for the writer (kept for potential graceful shutdown)
    writer_thread: Option<JoinHandle<()>>,
    /// Thread handle for the reader (kept for potential graceful shutdown)
    reader_thread: Option<JoinHandle<()>>,
}

impl PortConnection {
    pub fn new() -> Self {
        Self {
            writer_thread: None,
            reader_thread: None,
        }
    }

    /// Opens a port and spawns reader/writer threads.
    ///
    /// Returns a sender for writing data to the port.
    pub fn open(
        &mut self,
        name: Arc<str>,
        info: PortInfo,
        broadcast_channel: broadcast::Sender<Arc<PortEvent>>,
    ) -> Result<mpsc::Sender<Arc<Vec<u8>>>, SerialError> {
        let (writer_tx, writer_rx) = mpsc::channel();

        // Open port and create only the handles we need
        let handle = PortHandle::new().open(&info.path, info.baud_rate)?;
        let writer_handle = handle.try_clone()?;

        // Spawn writer thread (gets cloned handle)
        self.writer_thread = Some(Self::spawn_writer(writer_handle, writer_rx));

        // Spawn reader thread (gets original handle)
        self.reader_thread = Some(Self::spawn_reader(name, handle, broadcast_channel));

        Ok(writer_tx)
    }

    /// Spawns the reader thread that buffers incoming data and emits complete lines.
    fn spawn_reader(
        port_name: Arc<str>,
        mut reader_handle: PortHandle,
        broadcast: broadcast::Sender<Arc<PortEvent>>,
    ) -> JoinHandle<()> {
        thread::spawn(move || {
            let mut read_buf = [0u8; 1024];
            let mut line_buf = Vec::with_capacity(256);

            loop {
                match reader_handle.read(&mut read_buf) {
                    Ok(0) => continue, // Timeout, keep reading
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
                        // Flush remaining data before error
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

    /// Spawns the writer thread that sends data to the port.
    fn spawn_writer(
        mut port_handle: PortHandle,
        receiver: mpsc::Receiver<Arc<Vec<u8>>>,
    ) -> JoinHandle<()> {
        thread::spawn(move || {
            while let Ok(buf) = receiver.recv() {
                let _ = port_handle.write_all(buf.as_ref());
            }
        })
    }
}
