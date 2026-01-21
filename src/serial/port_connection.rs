use std::{
    sync::{
        Arc,
        mpsc::{self, Receiver},
    },
    thread::{self, JoinHandle},
};

use bytes::Bytes;
use tokio::sync::broadcast::{self};

use crate::{
    error::AppError,
    serial::{
        port_handle::{self, PortHandle},
        port_info::PortInfo,
    },
};

pub enum PortEvent {
    /// Data received from a port. Uses Arc<str> and Bytes for cheap cloning.
    Data {
        port: Arc<str>,
        data: Bytes,
    },
    Error(AppError),
    Disconnected(String),
    PortAdded(String),
    PortRemoved(String),
}

pub struct PortConnection {
    pub info: Option<PortInfo>,

    /// Handle to write to port
    writer_handle: Option<PortHandle>,
    /// Handle to read from port
    reader_handle: Option<PortHandle>,

    /// Channel for other components to write to port
    writer_channel: Option<Receiver<PortEvent>>,

    /// Thread to write to port
    writer_thread: Option<JoinHandle<()>>,
    /// Thread to read from port
    reader_thread: Option<JoinHandle<()>>,
}

impl PortConnection {
    // Spawns the reader and writer
    pub fn new() -> Self {
        Self {
            info: None,

            writer_handle: None,
            reader_handle: None,

            writer_channel: None,

            writer_thread: None,
            reader_thread: None,
        }
    }

    /// Start reading from port
    /// while reading, send to broadcast channel
    /// Components will have their own Sender in order to send
    /// string data to ports
    pub fn open(
        &mut self,
        name: Arc<str>,
        info: PortInfo,
        broadcast_channel: broadcast::Sender<Arc<PortEvent>>,
    ) -> Result<mpsc::Sender<Arc<Vec<u8>>>, AppError> {
        let (writer_tx, writer_rx) = mpsc::channel();

        // Open port and create only the handles we need
        let handle = PortHandle::new().open(&info.path, info.baud_rate)?;
        let writer_handle = handle.try_clone()?;

        // Spawn writer thread (gets cloned handle)
        self.writer_thread = Some(PortConnection::spawn_writer(writer_handle, writer_rx));

        // Spawn reader thread (gets original handle)
        self.reader_thread = Some(PortConnection::spawn_reader(
            name,
            handle,
            broadcast_channel,
        ));
        Ok(writer_tx)
    }

    pub fn close(self) -> Result<(), AppError> {
        if let Some(mut handle) = self.writer_handle {
            handle.close();
        }

        if let Some(mut handle) = self.reader_handle {
            handle.close();
        }
        Ok(())
    }

    /// Helper function to spawn and handle port reading.
    /// Buffers incoming data and emits complete lines (split on \n or \r).
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
                    Ok(0) => {
                        // Timeout or no data available - continue reading
                        continue;
                    }
                    Ok(n) => {
                        // Process each byte, emitting lines on \n or \r
                        for &byte in &read_buf[..n] {
                            if byte == b'\n' || byte == b'\r' {
                                // Skip empty lines (handles \r\n sequences)
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
                        // Emit any remaining buffered data before error
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

    /// Spawn writer thread for a particular port name
    fn spawn_writer(
        mut port_handle: PortHandle,
        receiver: Receiver<Arc<Vec<u8>>>,
    ) -> JoinHandle<()> {
        // Spawn a thread to read serial port
        // Move the port handle into here
        thread::spawn(move || {
            // While there is a connection to the writer keep thread
            while let Ok(buf) = receiver.recv() {
                let _ = port_handle.write_all(buf.as_ref());
            }
        })
    }
}
