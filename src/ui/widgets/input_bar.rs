//! Bottom input bar for typing commands to send.
//!
//! Shows a `[ports]` button on the left and text input on the right.
//! Supports modifier keys (Ctrl+Space to open send group).

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use super::focused_block;

/// Actions the input bar can request.
pub enum InputBarAction {
    /// Open the send group popup to select target ports
    OpenSendGroup,
    /// Send the text to selected ports
    Send(String),
}

/// Text input bar at the bottom of the screen.
///
/// Left side shows a clickable `[ports]` label, right side is
/// the text input area with a cursor indicator.
pub struct InputBar {
    /// Current input buffer
    input: String,
}

impl InputBar {
    /// Creates a new empty input bar.
    pub fn new() -> Self {
        Self {
            input: String::new(),
        }
    }

    /// Renders the input bar.
    ///
    /// Layout: `[ports] <input text>_`
    /// The underscore acts as a cursor indicator.
    pub fn render(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let block = focused_block(" Input ", focused);

        // Get inner area (inside borders)
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let port_sel_label = "[ports] ";

        // Port selection button
        let tab = Line::from(vec![Span::styled(
            port_sel_label,
            Style::default().fg(Color::Yellow),
        )]);

        // Split horizontally: label | input text
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(port_sel_label.len() as u16),
                Constraint::Min(1),
            ])
            .split(inner);

        frame.render_widget(Paragraph::new(tab), chunks[0]);

        // Input text with cursor
        let input_text = format!("{}_", self.input);
        frame.render_widget(Paragraph::new(input_text), chunks[1]);
    }

    /// Handles key input when this widget is focused.
    ///
    /// - `Ctrl+Space` -> Open send group popup
    /// - Characters -> Append to input
    /// - `Backspace` -> Delete last character
    /// - `Enter` -> Send input text (if not empty)
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<InputBarAction> {
        match (key.modifiers, key.code) {
            // Ctrl+Space opens the send group selector
            (KeyModifiers::CONTROL, KeyCode::Char(' ')) => Some(InputBarAction::OpenSendGroup),
            // Regular character input
            (_, KeyCode::Char(c)) => {
                self.input.push(c);
                None
            }
            // Backspace removes last character
            (_, KeyCode::Backspace) => {
                self.input.pop();
                None
            }
            // Enter sends the message
            (_, KeyCode::Enter) => {
                if !self.input.is_empty() {
                    let text = std::mem::take(&mut self.input);
                    Some(InputBarAction::Send(text))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}
