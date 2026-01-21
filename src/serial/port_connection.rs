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

pub enum PortEvent {
    Data(Vec<u8>),
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
        info: PortInfo,
        broadcast_channel: broadcast::Sender<Arc<PortEvent>>,
    ) -> Result<mpsc::Sender<Arc<Vec<u8>>>, AppError> {
        let (writer_tx, writer_rx) = mpsc::channel();

        // open a port handle
        let handle = PortHandle::new().open(&info.path, info.baud_rate)?;

        self.writer_handle = Some(handle.try_clone()?);
        self.reader_handle = Some(handle.try_clone()?);
        // Spawn writers
        self.writer_thread = Some(PortConnection::spawn_writer(handle.try_clone()?, writer_rx));

        // Spawn readers
        self.reader_thread = Some(PortConnection::spawn_reader(
            handle.try_clone()?,
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

    /// Helper function to spawn and handle port reading
    fn spawn_reader(
        mut reader_handle: PortHandle,
        broadcast: broadcast::Sender<Arc<PortEvent>>,
    ) -> JoinHandle<()> {
        // Spawn a thread to read serial port
        // Move the port handle into here
        thread::spawn(move || {
            // Buffer
            let buf = &mut [0; 1024];
            let disconn_buf = &mut [0; 64];
            loop {
                // read and send buffer
                match reader_handle.read(buf) {
                    Ok(0) => {
                        // Disconnected but retry
                        let _ = broadcast.send(Arc::new(PortEvent::Disconnected(
                            reader_handle.device_name().unwrap_or_default(),
                        )));
                        break;
                    }
                    Ok(n) => {
                        let _ = broadcast.send(Arc::new(PortEvent::Data(buf[..n].to_vec())));
                    }
                    // Break out of the thread if handle is gone
                    Err(e) => {
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
