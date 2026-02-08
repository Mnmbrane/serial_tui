//! Help popup showing available keybindings.
//!
//! Displays a scrollable list of all keyboard shortcuts grouped by
//! context (global, config bar, display, input bar, popups).

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use super::Popup;

/// Popup showing keyboard shortcuts and commands.
///
/// Static content, scrollable with j/k or arrow keys.
pub struct HelpPopup {
    popup: Popup,
    scroll: u16,
    pub visible: bool,
}

impl HelpPopup {
    pub fn new() -> Self {
        Self {
            popup: Popup::new(60, 70),
            scroll: 0,
            visible: false,
        }
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
        if self.visible {
            self.scroll = 0;
        }
    }

    pub fn render(&self, frame: &mut Frame) {
        if !self.visible {
            return;
        }

        let area = self.popup.area(frame.area());
        self.popup.clear(frame, area);

        let header = Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD);
        let key = Style::default().fg(Color::Cyan);
        let desc = Style::default().fg(Color::White);

        let lines = vec![
            Line::from(Span::styled("  Global", header)),
            Line::from(vec![
                Span::styled("    Tab       ", key),
                Span::styled("Cycle focus (Config → Display → Input)", desc),
            ]),
            Line::from(vec![
                Span::styled("    Esc       ", key),
                Span::styled("Quit application", desc),
            ]),
            Line::from(vec![
                Span::styled("    ?         ", key),
                Span::styled("Toggle this help screen", desc),
            ]),
            Line::from(""),
            Line::from(Span::styled("  Config Bar", header)),
            Line::from(vec![
                Span::styled("    p         ", key),
                Span::styled("Open port list", desc),
            ]),
            Line::from(vec![
                Span::styled("    a         ", key),
                Span::styled("Add new port", desc),
            ]),
            Line::from(""),
            Line::from(Span::styled("  Display", header)),
            Line::from(vec![
                Span::styled("    j / ↓     ", key),
                Span::styled("Scroll down", desc),
            ]),
            Line::from(vec![
                Span::styled("    k / ↑     ", key),
                Span::styled("Scroll up", desc),
            ]),
            Line::from(vec![
                Span::styled("    Ctrl+d    ", key),
                Span::styled("Half page down", desc),
            ]),
            Line::from(vec![
                Span::styled("    Ctrl+u    ", key),
                Span::styled("Half page up", desc),
            ]),
            Line::from(vec![
                Span::styled("    gg        ", key),
                Span::styled("Jump to top", desc),
            ]),
            Line::from(vec![
                Span::styled("    G         ", key),
                Span::styled("Jump to bottom", desc),
            ]),
            Line::from(vec![
                Span::styled("    v / V     ", key),
                Span::styled("Toggle visual selection", desc),
            ]),
            Line::from(vec![
                Span::styled("    y         ", key),
                Span::styled("Yank selection to clipboard", desc),
            ]),
            Line::from(vec![
                Span::styled("    /         ", key),
                Span::styled("Search", desc),
            ]),
            Line::from(vec![
                Span::styled("    n         ", key),
                Span::styled("Next search match", desc),
            ]),
            Line::from(vec![
                Span::styled("    N         ", key),
                Span::styled("Previous search match", desc),
            ]),
            Line::from(vec![
                Span::styled("    Enter     ", key),
                Span::styled("Focus input bar", desc),
            ]),
            Line::from(""),
            Line::from(Span::styled("  Input Bar", header)),
            Line::from(vec![
                Span::styled("    Ctrl+Space", key),
                Span::styled("  Open send target selector", desc),
            ]),
            Line::from(vec![
                Span::styled("    Enter     ", key),
                Span::styled("Send text to selected ports", desc),
            ]),
            Line::from(vec![
                Span::styled("    /clear    ", key),
                Span::styled("Clear display", desc),
            ]),
            Line::from(vec![
                Span::styled("    /purge    ", key),
                Span::styled("Purge log files", desc),
            ]),
            Line::from(""),
            Line::from(Span::styled("  Popups", header)),
            Line::from(vec![
                Span::styled("    j/k / ↑↓  ", key),
                Span::styled("Navigate items", desc),
            ]),
            Line::from(vec![
                Span::styled("    Enter/Space", key),
                Span::styled(" Select item", desc),
            ]),
            Line::from(vec![
                Span::styled("    Esc       ", key),
                Span::styled("Close popup", desc),
            ]),
        ];

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .title(" Help (? to close) ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::White)),
            )
            .scroll((self.scroll, 0));

        frame.render_widget(paragraph, area);
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('?') => {
                self.visible = false;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.scroll = self.scroll.saturating_add(1);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.scroll = self.scroll.saturating_sub(1);
            }
            _ => {}
        }
    }
}
