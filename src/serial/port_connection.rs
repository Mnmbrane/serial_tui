use std::{
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver, Sender},
    },
    thread::{self},
};

use crate::{
    error::AppError,
    serial::{port_handle::PortHandle, port_info::PortInfo},
};

pub enum PortEvent {
    Data(Vec<u8>),
    Error(AppError),
    Disconnected,
}

pub struct PortConnection {
    pub info: PortInfo,

    pub writer: Option<Sender<PortEvent>>,
    pub reader: Option<Receiver<PortEvent>>,
}

impl PortConnection {
    pub fn new(info: PortInfo) -> Result<Self, AppError> {
        let reader_handle = PortHandle::new().open(&info.path, info.baud_rate)?;
        let writer_handle = reader_handle.try_clone()?;

        let reader = PortConnection::spawn_reader(reader_handle);
        let writer = PortConnection::spawn_writer(writer_handle);

        Ok(Self {
            info: info.clone(),
            writer: None,
            reader: None,
        })
    }

    /// Spawn reader thread for a particular port name
    fn spawn_reader(mut port_handle: PortHandle) -> Receiver<PortEvent> {
        // Start thread if open
        let (tx, rx) = mpsc::channel();
        // Spawn a thread to read serial port
        // Move the port handle into here
        thread::spawn(move || {
            // Buffer
            let buf = &mut [0; 1024];
            loop {
                // read and send buffer
                match port_handle.read(buf) {
                    Ok(0) => {
                        // Disconnected but retry
                        tx.send(PortEvent::Disconnected);
                        break;
                    }
                    Ok(n) => {
                        tx.send(PortEvent::Data(buf[..n].to_vec()));
                    }
                    // Break out of the thread if handle is gone
                    Err(e) => {
                        tx.send(PortEvent::Error(e));
                        break;
                    }
                }
            }
        });

        rx
    }

    /// Spawn writer thread for a particular port name
    fn spawn_writer(mut port_handle: PortHandle) -> Sender<PortEvent> {
        // Start thread if open
        let (tx, rx) = mpsc::channel();
        // Spawn a thread to read serial port
        // Move the port handle into here
        thread::spawn(move || {
            // While there is a connection to the writer keep thread
            while let Ok(port_event) = rx.recv() {
                if let PortEvent::Data(buf) = port_event {
                    port_handle.write_all(buf.as_slice());
                }
            }
        });

        tx
    }

    pub fn close(&self) {}
}
