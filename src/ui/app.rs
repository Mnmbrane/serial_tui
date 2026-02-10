//! Main UI orchestrator.
//!
//! Owns all widgets and popups, handles the render loop, and routes
//! keyboard input to the appropriate component based on focus and
//! popup visibility.

use std::io;

use anyhow::Result;
use bytes::Bytes;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind,
    },
    execute,
};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
};
use std::sync::mpsc;

use crate::{
    logger::LoggerEvent,
    serial::{PortEvent, hub::SerialHub},
    ui::{
        HelpPopup, PortListPopup, SendGroupPopup, UiEvent,
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
    hub: SerialHub,
    /// Receiver for UI events from background components
    ui_rx: mpsc::Receiver<UiEvent>,
    /// Sender for logger events
    log_tx: mpsc::Sender<LoggerEvent>,

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
    /// Modal popup showing keyboard shortcuts
    help_popup: HelpPopup,

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
    /// initializes all widgets with default state. All ports are
    /// selected for sending by default.
    pub fn new(
        hub: SerialHub,
        ui_rx: mpsc::Receiver<UiEvent>,
        log_tx: mpsc::Sender<LoggerEvent>,
    ) -> Self {
        let mut send_group_popup = SendGroupPopup::new();
        send_group_popup.select_all(&hub.list_ports());

        Self {
            hub,
            ui_rx,
            log_tx,
            config_bar: ConfigBar,
            display: Display::new(),
            input_bar: InputBar::new(),
            port_list_popup: PortListPopup::new(),
            send_group_popup,
            notification_popup: Notification::new(),
            help_popup: HelpPopup::new(),
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
    pub fn run(&mut self) -> Result<()> {
        let mut terminal = ratatui::init();
        execute!(io::stdout(), EnableMouseCapture)?;

        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }

        execute!(io::stdout(), DisableMouseCapture)?;
        ratatui::restore();
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
        let ports = self.hub.list_ports();

        if self.port_list_popup.visible {
            self.port_list_popup.render(frame, &ports);
        }

        if self.send_group_popup.visible {
            self.send_group_popup.render(frame, &ports);
        }

        if self.help_popup.visible {
            self.help_popup.render(frame);
        }

        if self.notification_popup.is_visible() {
            self.notification_popup.render(frame);
        }
    }

    /// Polls for and handles input events.
    ///
    /// Uses 16ms timeout (~60fps) for responsive UI updates.
    /// Only processes key press events (ignores key release).
    pub fn handle_events(&mut self) -> Result<()> {
        while let Ok(event) = self.ui_rx.try_recv() {
            match event {
                UiEvent::PortData(port_event) => {
                    let PortEvent {
                        port,
                        data,
                        timestamp,
                    } = port_event.as_ref();
                    let timestamp = timestamp.format("%H:%M:%S%.3f");
                    let text = String::from_utf8_lossy(data);

                    // Look up port color from config
                    let port_color = self
                        .hub
                        .get_config(port)
                        .map(|info| info.color.0)
                        .unwrap_or(Color::Reset);

                    // Build styled line with colored port name
                    let line = Line::from(vec![
                        Span::raw(format!("[{timestamp}] ")),
                        Span::styled(format!("[{port}]"), Style::default().fg(port_color)),
                        Span::raw(format!(" {text}")),
                    ]);
                    self.display.push_line(line);
                }
                UiEvent::ShowNotification(msg) => {
                    self.notification_popup.show(msg.to_string());
                }
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
        let ports = self.hub.list_ports();

        // Popups capture all input when visible
        if self.help_popup.visible {
            self.help_popup.handle_key(key);
            return;
        }

        if self.port_list_popup.visible {
            self.port_list_popup.handle_key(key, &ports);
            return;
        }

        if self.send_group_popup.visible {
            self.send_group_popup.handle_key(key, &ports);
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
            KeyCode::Char('?') => {
                self.help_popup.toggle();
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
                    }
                }
            }
            Focus::Display => {
                if let Some(action) = self.display.handle_key(key, self.display_height) {
                    match action {
                        DisplayAction::FocusInput => {
                            self.focus = Focus::InputBar;
                        }
                        DisplayAction::Notify(msg) => {
                            self.notification_popup.show(msg);
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
                        InputBarAction::Send(text) => match text.as_str() {
                            "/clear" => self.display.clear(),
                            "/help" => self.help_popup.toggle(),
                            "/purge" => {
                                let _ = self.log_tx.send(LoggerEvent::Purge);
                            }
                            _ => {
                                let selected = self.send_group_popup.get_selected();
                                if selected.is_empty() {
                                    self.notification_popup.show("No ports selected");
                                } else {
                                    if let Err(e) = self.hub.send(&selected, Bytes::from(text)) {
                                        self.notification_popup.show(format!("Send failed: {e}"));
                                    }
                                }
                            }
                        },
                    }
                }
            }
        }
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
