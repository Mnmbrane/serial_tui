//! Port list popup for viewing connected ports.
//!
//! Shows all configured ports with their status (connected indicator)
//! and baud rate. Arrow keys navigate, Enter selects.

use std::sync::Arc;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
};

use crate::config::PortConfig;

use super::{Popup, select_next, select_prev};

/// Popup showing list of available COM ports.
///
/// Stateless regarding port data - receives it during render/handle_key
/// to stay synchronized with the serial manager. Maintains only UI state
/// (selection index, visibility).
pub struct PortListPopup {
    /// Helper for centered positioning
    popup: Popup,
    /// Current selection in the list
    list_state: ListState,
    /// Whether the popup is currently shown
    pub visible: bool,
}

impl PortListPopup {
    /// Creates a new hidden port list popup.
    ///
    /// Uses 40% width, 50% height of the screen.
    pub fn new() -> Self {
        Self {
            popup: Popup::new(40, 50),
            list_state: ListState::default().with_selected(Some(0)),
            visible: false,
        }
    }

    /// Toggles visibility, resetting selection on open.
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
        if self.visible {
            self.list_state.select(Some(0));
        }
    }

    /// Renders the port list.
    ///
    /// Shows each port with a connection indicator (●), name, and baud rate.
    /// Current selection is highlighted with a dark background.
    pub fn render(&mut self, frame: &mut Frame, ports: &[(Arc<str>, Arc<PortConfig>)]) {
        if !self.visible {
            return;
        }

        let area = self.popup.area(frame.area());
        self.popup.clear(frame, area);

        // Build list items: "● port_name  baud_rate"
        let items: Vec<ListItem> = ports
            .iter()
            .map(|(name, info)| {
                let line = Line::from(vec![
                    Span::styled("● ", Style::default().fg(Color::Green)),
                    Span::raw(format!("{}  {}", name, info.baud_rate)),
                ]);
                ListItem::new(line)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title(" Ports ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::White)),
            )
            .highlight_style(Style::default().bg(Color::DarkGray));

        frame.render_stateful_widget(list, area, &mut self.list_state);
    }

    /// Handles key input when this popup is visible.
    ///
    /// - `Esc` -> Close popup
    /// - `Up/k` -> Select previous
    /// - `Down/j` -> Select next
    /// - `Enter` -> Select current port
    pub fn handle_key(&mut self, key: KeyEvent, ports: &[(Arc<str>, Arc<PortConfig>)]) {
        match key.code {
            KeyCode::Esc => self.visible = false,
            KeyCode::Up | KeyCode::Char('k') => select_prev(&mut self.list_state, ports.len()),
            KeyCode::Down | KeyCode::Char('j') => select_next(&mut self.list_state, ports.len()),
            _ => {}
        }
    }

}
