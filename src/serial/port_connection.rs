//! Manages a single serial port connection with background read/write threads.
//!
//! Each `PortConnection` spawns two threads:
//! - A reader thread that continuously reads from the port and broadcasts data
//! - A writer thread that receives data via channel and writes to the port

use std::{
    sync::{
        Arc,
        mpsc::{self, Receiver},
    },
    thread::{self, JoinHandle},
};

use tokio::sync::broadcast::{self};

use crate::{
    error::AppError,
    serial::{
        port_handle::{self, PortHandle},
        port_info::PortInfo,
    },
};

/// Events emitted by serial port connections.
///
/// Broadcast to subscribers when data is received, errors occur,
/// or the port state changes.
pub enum PortEvent {
    /// Data received from the serial port
    Data(Vec<u8>),
    /// Error occurred during read/write
    Error(AppError),
    /// Port disconnected (EOF or device removed)
    Disconnected,
    /// A new port was added to the manager
    PortAdded(String),
    /// A port was removed from the manager
    PortRemoved(String),
}

/// Manages a single serial port with background I/O threads.
///
/// Owns the port handles and spawned threads for reading/writing.
/// Data is received via broadcast channel, sent via mpsc channel.
pub struct PortConnection {
    /// Port configuration (if set)
    pub info: Option<PortInfo>,

    /// Handle used by the writer thread
    writer_handle: Option<PortHandle>,
    /// Handle used by the reader thread
    reader_handle: Option<PortHandle>,

    /// Receives data to write (unused currently, writer_rx passed to thread)
    writer_channel: Option<Receiver<PortEvent>>,

    /// Join handle for the writer thread
    writer_thread: Option<JoinHandle<()>>,
    /// Join handle for the reader thread
    reader_thread: Option<JoinHandle<()>>,
}

impl PortConnection {
    /// Creates a new uninitialized port connection.
    ///
    /// Call `open()` to actually connect to a serial port.
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

    /// Opens the serial port and spawns reader/writer threads.
    ///
    /// The reader thread broadcasts received data to `broadcast_channel`.
    /// Returns a sender for writing data to the port.
    ///
    /// # Arguments
    /// * `info` - Port configuration (path, baud rate, etc.)
    /// * `broadcast_channel` - Channel to broadcast received data and events
    ///
    /// # Returns
    /// A sender that can be used to write data to this port
    pub fn open(
        &mut self,
        info: PortInfo,
        broadcast_channel: broadcast::Sender<Arc<PortEvent>>,
    ) -> Result<mpsc::Sender<Arc<Vec<u8>>>, AppError> {
        let (writer_tx, writer_rx) = mpsc::channel();

        // Open the underlying port handle
        let handle = PortHandle::new().open(&info.path, info.baud_rate)?;

        // Clone handles for reader and writer threads
        self.writer_handle = Some(handle.try_clone()?);
        self.reader_handle = Some(handle.try_clone()?);

        // Spawn background threads
        self.writer_thread = Some(PortConnection::spawn_writer(handle.try_clone()?, writer_rx));
        self.reader_thread = Some(PortConnection::spawn_reader(
            handle.try_clone()?,
            broadcast_channel,
        ));

        Ok(writer_tx)
    }

    /// Closes the port connection by closing both handles.
    ///
    /// This will cause the reader/writer threads to terminate.
    pub fn close(self) -> Result<(), AppError> {
        if let Some(mut handle) = self.writer_handle {
            handle.close();
        }

        if let Some(mut handle) = self.reader_handle {
            handle.close();
        }
        Ok(())
    }

    /// Spawns a background thread that continuously reads from the port.
    ///
    /// Broadcasts `PortEvent::Data` for each successful read,
    /// `PortEvent::Disconnected` on EOF, and `PortEvent::Error` on failure.
    /// Thread exits on disconnect or error.
    fn spawn_reader(
        mut reader_handle: PortHandle,
        broadcast: broadcast::Sender<Arc<PortEvent>>,
    ) -> JoinHandle<()> {
        thread::spawn(move || {
            let buf = &mut [0; 1024];
            loop {
                match reader_handle.read(buf) {
                    Ok(0) => {
                        let _ = broadcast.send(Arc::new(PortEvent::Disconnected));
                        break;
                    }
                    Ok(n) => {
                        let _ = broadcast.send(Arc::new(PortEvent::Data(buf[..n].to_vec())));
                    }
                    Err(e) => {
                        let _ = broadcast.send(Arc::new(PortEvent::Error(e)));
                        break;
                    }
                }
            }
        })
    }

    /// Spawns a background thread that writes data received via channel.
    ///
    /// Loops until the sender is dropped (channel closed).
    /// Writes each received buffer to the port.
    fn spawn_writer(
        mut port_handle: PortHandle,
        receiver: Receiver<Arc<Vec<u8>>>,
    ) -> JoinHandle<()> {
        thread::spawn(move || {
            while let Ok(buf) = receiver.recv() {
                let _ = port_handle.write_all(buf.as_ref());
            }
        })
    }
}
