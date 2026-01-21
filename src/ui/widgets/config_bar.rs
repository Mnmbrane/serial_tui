//! Top configuration bar widget.
//!
//! Displays keybinding hints for port operations: [p]orts and [a]dd.

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use super::focused_block;

/// Actions the config bar can request.
pub enum ConfigAction {
    /// Show a notification message
    Notify(String),
    /// Open the port list popup
    OpenPorts,
    /// Open the add port dialog
    AddPort,
}

/// Top bar showing port control keybindings.
pub struct ConfigBar;

impl ConfigBar {
    /// Creates a new config bar.
    pub fn new() -> Self {
        Self {}
    }

    /// Renders the config bar with keybinding hints.
    ///
    /// Shows `[p]orts  [a]dd` with highlighted key letters.
    pub fn render(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let block = focused_block(" Config ", focused);

        let content = Line::from(vec![
            Span::styled("p", Style::default().fg(Color::Yellow)),
            Span::styled("orts", Style::default().fg(Color::Cyan)),
            Span::raw("  "),
            Span::styled("a", Style::default().fg(Color::Yellow)),
            Span::styled("dd", Style::default().fg(Color::Green)),
        ]);

        let paragraph = Paragraph::new(content).block(block);
        frame.render_widget(paragraph, area);
    }

    /// Handles key input when this widget is focused.
    ///
    /// - `p` -> Open ports list
    /// - `a` -> Add new port
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<ConfigAction> {
        match key.code {
            KeyCode::Char('p') => Some(ConfigAction::OpenPorts),
            KeyCode::Char('a') => Some(ConfigAction::AddPort),
            _ => None,
        }
    }
}
