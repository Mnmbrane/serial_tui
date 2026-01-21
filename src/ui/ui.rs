//! Main UI orchestrator.
//!
//! Owns all widgets and popups, handles the render loop, and routes
//! keyboard input to the appropriate component based on focus and
//! popup visibility.

use std::{fmt::format, io, sync::Arc};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout},
};
use serde::de::Error;
use tokio::sync::broadcast;

use crate::{
    error::AppError,
    serial::{port_connection::PortEvent, serial_manager::SerialManager},
    ui::{
        PortListAction, PortListPopup, SendGroupAction, SendGroupPopup,
        popup::Notification,
        widgets::{ConfigAction, DisplayAction, InputBarAction},
    },
};

use super::widgets::{ConfigBar, Display, InputBar};

/// Which widget currently has keyboard focus.
#[derive(PartialEq, Clone, Copy)]
pub enum Focus {
    ConfigBar,
    Display,
    InputBar,
}

/// Main UI state container.
///
/// Holds references to the serial manager and all UI components.
/// The render loop runs in `run()`, polling for keyboard events
/// and redrawing the screen each frame.
pub struct Ui {
    /// Reference to the serial manager for port operations
    serial_manager: Arc<SerialManager>,
    /// Receiver for serial port events (data, errors, etc.)
    serial_rx: broadcast::Receiver<Arc<PortEvent>>,

    /// Top bar showing port controls
    config_bar: ConfigBar,
    /// Main area for serial output display
    display: Display,
    /// Bottom bar for text input
    input_bar: InputBar,

    /// Modal popup for port list selection
    port_list_popup: PortListPopup,
    /// Modal popup for selecting send targets
    send_group_popup: SendGroupPopup,
    /// Toast notification overlay
    notification_popup: Notification,

    /// Currently focused widget
    focus: Focus,

    /// Cached display height for key handling
    display_height: usize,

    /// Set to true to exit the application
    exit: bool,
}

impl Ui {
    /// Creates a new UI with the given serial manager.
    ///
    /// Subscribes to the serial manager's broadcast channel and
    /// initializes all widgets with default state.
    pub fn new(serial_manager: Arc<SerialManager>) -> Self {
        let serial_rx = serial_manager.subscribe();
        Self {
            serial_manager,
            serial_rx,
            config_bar: ConfigBar::new(),
            display: Display::new(),
            input_bar: InputBar::new(),
            port_list_popup: PortListPopup::new(),
            send_group_popup: SendGroupPopup::new(),
            notification_popup: Notification::new(),
            focus: Focus::InputBar,
            display_height: 0,
            exit: false,
        }
    }

    /// Starts the main render loop.
    ///
    /// Enters raw mode, switches to alternate screen, and runs the
    /// draw/event loop until `exit` is set to true. Restores terminal
    /// state on exit.
    pub fn run(&mut self) -> Result<(), AppError> {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;

        let mut terminal = ratatui::init();

        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }

        // Restore terminal state
        disable_raw_mode()?;
        execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
        Ok(())
    }

    /// Renders all UI components to the frame.
    ///
    /// Layout: ConfigBar (top, 3 lines) | Display (middle, flex) | InputBar (bottom, 3 lines)
    /// Popups are rendered on top if visible.
    pub fn draw(&mut self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // config bar
                Constraint::Min(5),    // display
                Constraint::Length(3), // input bar
            ])
            .split(frame.area());

        // Cache display height for key handling
        self.display_height = chunks[1].height.saturating_sub(2) as usize; // minus borders

        // Render main widgets with focus indication
        self.config_bar
            .render(frame, chunks[0], self.focus == Focus::ConfigBar);
        self.display
            .render(frame, chunks[1], self.focus == Focus::Display);
        self.input_bar
            .render(frame, chunks[2], self.focus == Focus::InputBar);

        // Render popups on top (pass current port list)
        let ports = self.serial_manager.get_port_list();

        if self.port_list_popup.visible {
            self.port_list_popup.render(frame, &ports);
        }

        if self.send_group_popup.visible {
            self.send_group_popup.render(frame, &ports);
        }

        if self.notification_popup.is_visible() {
            self.notification_popup.render(frame);
        }
    }

    /// Polls for and handles input events.
    ///
    /// Uses 16ms timeout (~60fps) for responsive UI updates.
    /// Only processes key press events (ignores key release).
    pub fn handle_events(&mut self) -> Result<(), AppError> {
        while let Ok(event) = self.serial_rx.try_recv() {
            match event.as_ref() {
                PortEvent::Data(items) => {
                    let text = String::from_utf8_lossy(items).into_owned();
                    self.display.push_line(text);
                }
                PortEvent::Error(app_error) => {
                    self.notification_popup
                        .show(format!("Error: {}", app_error));
                }
                PortEvent::Disconnected(port_name) => self
                    .notification_popup
                    .show(format!("Disconnected {}", port_name)),
                PortEvent::PortAdded(port_name) => self
                    .notification_popup
                    .show(format!("Port Added {}", port_name)),
                PortEvent::PortRemoved(port_name) => self
                    .notification_popup
                    .show(format!("Port Removed {}", port_name)),
            }
        }
        if event::poll(std::time::Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    self.handle_key(key);
                }
            }
        }
        Ok(())
    }

    /// Routes keyboard input to the appropriate handler.
    ///
    /// Priority: Visible popups > Global keys (Esc, Tab) > Focused widget.
    /// Widgets return actions that are processed here.
    fn handle_key(&mut self, key: KeyEvent) {
        let ports = self.serial_manager.get_port_list();

        // Popups capture all input when visible
        if self.port_list_popup.visible {
            if let Some(action) = self.port_list_popup.handle_key(key, &ports) {
                match action {
                    PortListAction::Select(name) => {
                        // TODO: handle port selection
                    }
                    PortListAction::Close => {}
                }
            }
            return;
        }

        if self.send_group_popup.visible {
            if let Some(action) = self.send_group_popup.handle_key(key, &ports) {
                match action {
                    SendGroupAction::Close => {}
                }
            }
            return;
        }

        // Global keys (always available when no popup)
        match key.code {
            KeyCode::Esc => {
                self.exit = true;
                return;
            }
            KeyCode::Tab => {
                self.cycle_focus();
                return;
            }
            _ => {}
        }

        // Route to focused widget, handle returned actions
        match self.focus {
            Focus::ConfigBar => {
                if let Some(action) = self.config_bar.handle_key(key) {
                    match action {
                        ConfigAction::OpenPorts => self.port_list_popup.toggle(),
                        ConfigAction::AddPort => { /* TODO */ }
                        ConfigAction::Notify(msg) => self.notification_popup.show(msg),
                    }
                }
            }
            Focus::Display => {
                if let Some(action) = self.display.handle_key(key, self.display_height) {
                    match action {
                        DisplayAction::FocusInput => {
                            self.set_focus(Focus::InputBar);
                        }
                    }
                }
            }
            Focus::InputBar => {
                if let Some(action) = self.input_bar.handle_key(key) {
                    match action {
                        InputBarAction::OpenSendGroup => {
                            self.send_group_popup.toggle();
                        }
                        InputBarAction::Send(text) => {
                            let selected = self.send_group_popup.get_selected();
                            if selected.is_empty() {
                                self.notification_popup.show("No ports selected");
                            } else {
                                let _ = self.serial_manager.send(&selected, text.into_bytes());
                                self.notification_popup
                                    .show(format!("Sent to {} port(s)", selected.len()));
                            }
                        }
                    }
                }
            }
        }
    }

    /// Sets focus to the specified widget.
    fn set_focus(&mut self, focus: Focus) {
        self.focus = focus;
    }

    /// Cycles focus to the next widget in order.
    ///
    /// Order: ConfigBar -> Display -> InputBar -> ConfigBar
    fn cycle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::ConfigBar => Focus::Display,
            Focus::Display => Focus::InputBar,
            Focus::InputBar => Focus::ConfigBar,
        };
    }
}
