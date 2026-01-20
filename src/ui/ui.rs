use std::{
    io,
    sync::{Arc, mpsc::Receiver},
};

use crossterm::{
    event::EnableMouseCapture,
    execute,
    terminal::{EnterAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{DefaultTerminal, Frame};
use tokio::sync::broadcast;

use crate::{
    error::AppError,
    serial::{port_connection::PortEvent, serial_manager::SerialManager},
};

pub struct Ui {
    /// Used to send/receive serial data
    serial_manager: Arc<SerialManager>,
    serial_rx: broadcast::Receiver<Arc<PortEvent>>,

    /// Exit flag
    exit: bool,
}

impl Ui {
    pub fn new(serial_manager: Arc<SerialManager>) -> Self {
        let serial_rx = serial_manager.subscribe();
        Self {
            serial_manager,
            serial_rx,
            exit: false,
        }
    }

    /// Helper function to send a message to a list of com ports
    fn send_serial(&self, port_list: &[String], message: String) {
        self.serial_manager
            .send(port_list, message.as_bytes().to_vec());
    }

    pub fn run(&mut self) -> Result<(), AppError> {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;

        ratatui::run(|terminal| -> Result<(), AppError> {
            while !self.exit {
                terminal.draw(|frame| self.draw(frame))?;
            }
            Ok(())
        })?;
        Ok(())
    }

    pub fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    pub fn handle_events(&mut self) -> Result<(), AppError> {
        Ok(())
    }
}
