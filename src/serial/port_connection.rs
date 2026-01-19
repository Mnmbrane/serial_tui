use std::{
    sync::mpsc::{self, Receiver, Sender},
    thread::{self, JoinHandle},
};

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
    Disconnected,
}

pub struct PortConnection {
    pub info: Option<PortInfo>,

    writer_handle: Option<PortHandle>,
    reader_handle: Option<PortHandle>,

    writer_channel: Option<Sender<PortEvent>>,
    reader_channel: Option<Receiver<PortEvent>>,

    writer_thread: Option<JoinHandle<()>>,
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
            reader_channel: None,

            writer_thread: None,
            reader_thread: None,
        }
    }

    /// Multiple Producers / Single Consumer
    /// Multiple writes / single read
    /// Writer channel needs to read
    pub fn open(
        &mut self,
        info: PortInfo,
    ) -> Result<(Sender<PortEvent>, Receiver<PortEvent>), AppError> {
        let (writer_tx, writer_rx) = mpsc::channel();
        let (reader_tx, reader_rx) = mpsc::channel();

        let handle = PortHandle::new().open(&info.path, info.baud_rate)?;

        self.writer_handle = Some(handle.try_clone()?);
        self.reader_handle = Some(handle.try_clone()?);
        // Spawn writers
        self.writer_thread = Some(PortConnection::spawn_writer(handle.try_clone()?, writer_rx));

        // Spawn readers
        self.reader_thread = Some(PortConnection::spawn_reader(handle.try_clone()?, reader_tx));
        Ok((writer_tx, reader_rx))
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

    /// Spawn reader thread for a particular port name
    fn spawn_reader(mut reader_handle: PortHandle, sender: Sender<PortEvent>) -> JoinHandle<()> {
        // Spawn a thread to read serial port
        // Move the port handle into here
        thread::spawn(move || {
            // Buffer
            let buf = &mut [0; 1024];
            loop {
                // read and send buffer
                match reader_handle.read(buf) {
                    Ok(0) => {
                        // Disconnected but retry
                        sender.send(PortEvent::Disconnected);
                        break;
                    }
                    Ok(n) => {
                        sender.send(PortEvent::Data(buf[..n].to_vec()));
                    }
                    // Break out of the thread if handle is gone
                    Err(e) => {
                        sender.send(PortEvent::Error(e));
                        break;
                    }
                }
            }
        })
    }

    /// Spawn writer thread for a particular port name
    fn spawn_writer(mut port_handle: PortHandle, receiver: Receiver<PortEvent>) -> JoinHandle<()> {
        // Spawn a thread to read serial port
        // Move the port handle into here
        thread::spawn(move || {
            // While there is a connection to the writer keep thread
            while let Ok(port_event) = receiver.recv() {
                if let PortEvent::Data(buf) = port_event {
                    port_handle.write_all(buf.as_slice());
                }
            }
        })
    }
}
