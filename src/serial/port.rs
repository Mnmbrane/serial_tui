//! Single port connection with reader/writer threads.

use std::{
    io::{Read, Write},
    sync::{
        Arc,
        mpsc::{self, Receiver, Sender},
    },
    time::Duration,
};

use bytes::{Bytes, BytesMut};
use chrono::{DateTime, Local};
use memchr::{
    memchr,
    memmem::{self},
};
use serialport::SerialPort;

use crate::{
    config::{PortConfig, port::LineEnding},
    logger::LoggerEvent,
    serial::SerialError,
    ui::UiEvent,
};

/// Event emitted when a serial port receives data.
pub struct PortEvent {
    pub port: Arc<str>,
    pub data: Bytes,
    pub timestamp: DateTime<Local>,
}

/// A connected serial port with running reader/writer threads.
pub struct Port {
    pub writer_tx: mpsc::SyncSender<Bytes>,
    pub config: Arc<PortConfig>,
}

impl Port {
    fn find_delim(buf: &[u8], le: &LineEnding) -> Option<usize> {
        match le {
            LineEnding::Lf => memchr(b'\n', buf),
            LineEnding::Cr => memchr(b'\r', buf),
            LineEnding::CrLf => memmem::find(buf, b"\r\n"),
        }
    }

    fn spawn_reader(
        mut port: Box<dyn SerialPort>,
        name: Arc<str>,
        ui_tx: Sender<UiEvent>,
        log_tx: Sender<LoggerEvent>,
        line_ending: LineEnding,
    ) {
        std::thread::spawn(move || {
            let mut tmp_buf = [0; 4096];
            let mut accum = BytesMut::new();
            loop {
                // Read data
                let read_data_len = match port.read(&mut tmp_buf) {
                    Ok(0) => break,
                    Ok(n) => n,
                    Err(e)
                        if e.kind() == std::io::ErrorKind::TimedOut
                            || e.kind() == std::io::ErrorKind::WouldBlock =>
                    {
                        continue;
                    }
                    Err(e) => {
                        let _ = ui_tx.send(UiEvent::ShowNotification(
                            format!("{name}: read error: {e}").into(),
                        ));
                        break;
                    }
                };

                // Add to the accumulator including the line ending
                accum.extend_from_slice(&tmp_buf[..read_data_len]);

                // Check if accumulator has the line ending
                while let Some(delim_index) = Port::find_delim(&accum, &line_ending) {
                    let data = accum.split_to(delim_index + line_ending.len());

                    // only take payload not the line ending
                    let data = data.freeze();

                    // Send data to UI and logger
                    let port_event = PortEvent {
                        port: name.clone(),
                        data,
                        timestamp: Local::now(),
                    };

                    let event = Arc::new(port_event);
                    let _ = log_tx.send(LoggerEvent::SerialData(event.clone()));

                    if ui_tx.send(UiEvent::PortData(event)).is_err() {
                        break; // receiver dropped
                    }
                }
            }
        });
    }

    fn spawn_writer(
        mut port: Box<dyn SerialPort>,
        name: Arc<str>,
        writer_rx: Receiver<Bytes>,
        ui_tx: Sender<UiEvent>,
    ) {
        std::thread::spawn(move || {
            while let Ok(data) = writer_rx.recv() {
                if let Err(e) = port.write_all(&data) {
                    let _ = ui_tx.send(UiEvent::ShowNotification(
                        format!("{name}: write error: {e}").into(),
                    ));
                    break;
                }
            }
        });
    }
    /// Opens a port and spawns rea der/writer threads.
    pub fn open(
        name: Arc<str>,
        config: PortConfig,
        ui_tx: Sender<UiEvent>,
        log_tx: Sender<LoggerEvent>,
    ) -> Result<Self, SerialError> {
        let port = serialport::new(config.path.to_string_lossy(), config.baud_rate)
            .timeout(Duration::from_millis(10))
            .open()?;

        // Spawn reader thread
        Port::spawn_reader(
            port.try_clone()?,
            name.clone(),
            ui_tx.clone(),
            log_tx.clone(),
            config.line_ending,
        );

        let (writer_tx, writer_rx) = mpsc::sync_channel::<Bytes>(32);
        Port::spawn_writer(port, name, writer_rx, ui_tx);

        Ok(Port {
            writer_tx,
            config: Arc::new(config),
        })
    }
}
