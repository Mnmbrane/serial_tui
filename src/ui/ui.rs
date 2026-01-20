use std::{io, sync::Arc};

use crossterm::{
    event::{self, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout},
};
use tokio::sync::broadcast;

use crate::{
    error::AppError,
    serial::{port_connection::PortEvent, serial_manager::SerialManager},
    ui::{PortListPopup, widgets::ConfigAction},
};

use super::widgets::{ConfigBar, Display, InputBar};

#[derive(PartialEq, Clone, Copy)]
pub enum Focus {
    ConfigBar,
    Display,
    InputBar,
}

pub struct Ui {
    /// Used to send/receive serial data
    serial_manager: Arc<SerialManager>,
    serial_rx: broadcast::Receiver<Arc<PortEvent>>,

    /// Widgets
    config_bar: ConfigBar,
    display: Display,
    input_bar: InputBar,

    /// Popups
    port_list_popup: PortListPopup,

    /// Focus state
    focus: Focus,

    /// Exit flag
    exit: bool,
}

impl Ui {
    pub fn new(serial_manager: Arc<SerialManager>) -> Self {
        let serial_rx = serial_manager.subscribe();
        Self {
            serial_manager,
            serial_rx,
            config_bar: ConfigBar::new(),
            display: Display::new(),
            input_bar: InputBar::new(),
            port_list_popup: PortListPopup::new(),
            focus: Focus::InputBar,
            exit: false,
        }
    }

    pub fn run(&mut self) -> Result<(), AppError> {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;

        let mut terminal = ratatui::init();

        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }

        ratatui::restore();
        Ok(())
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // config bar
                Constraint::Min(5),    // display
                Constraint::Length(3), // input bar
            ])
            .split(frame.area());

        // Render all the widgets
        self.config_bar
            .render(frame, chunks[0], self.focus == Focus::ConfigBar);
        self.display
            .render(frame, chunks[1], self.focus == Focus::Display);
        self.input_bar
            .render(frame, chunks[2], self.focus == Focus::InputBar);

        // Render popups
        self.port_list_popup.render(frame);
    }

    pub fn handle_events(&mut self) -> Result<(), AppError> {
        if event::poll(std::time::Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    self.handle_key(key);
                }
            }
        }
        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => self.exit = true,
            KeyCode::Tab => self.cycle_focus(),
            _ => match self.focus {
                Focus::ConfigBar => match self.config_bar.handle_key(key) {
                    Some(ConfigAction::OpenPorts) => {
                        self.port_list_popup.toggle();
                    }
                    Some(ConfigAction::AddPort) => todo!(),
                    None => (),
                },
                Focus::Display => self.display.handle_key(key),
                Focus::InputBar => self.input_bar.handle_key(key),
            },
        }
    }

    fn cycle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::ConfigBar => Focus::Display,
            Focus::Display => Focus::InputBar,
            Focus::InputBar => Focus::ConfigBar,
        };
    }
}
