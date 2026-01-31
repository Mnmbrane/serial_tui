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

use crate::serial::config::PortConfig;

use super::Popup;

/// Actions returned by the port list popup.
pub enum PortListAction {
    /// User selected a port by name
    Select(String),
    /// Popup was closed (Esc pressed)
    Close,
}

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

    /// Opens the popup and selects the first item.
    #[allow(dead_code)]
    pub fn open(&mut self) {
        self.visible = true;
        self.list_state.select(Some(0));
    }

    /// Closes the popup.
    #[allow(dead_code)]
    pub fn close(&mut self) {
        self.visible = false;
    }

    /// Renders the port list.
    ///
    /// Shows each port with a connection indicator (●), name, and baud rate.
    /// Current selection is highlighted with a dark background.
    pub fn render(&mut self, frame: &mut Frame, ports: &[(String, Arc<PortConfig>)]) {
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
    pub fn handle_key(
        &mut self,
        key: KeyEvent,
        ports: &[(String, Arc<PortConfig>)],
    ) -> Option<PortListAction> {
        match key.code {
            KeyCode::Esc => {
                self.visible = false;
                Some(PortListAction::Close)
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.select_prev(ports.len());
                None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.select_next(ports.len());
                None
            }
            KeyCode::Enter => {
                if let Some(i) = self.list_state.selected() {
                    if let Some((name, _)) = ports.get(i) {
                        return Some(PortListAction::Select(name.clone()));
                    }
                }
                None
            }
            _ => None,
        }
    }

    /// Moves selection to the next item (wraps around).
    fn select_next(&mut self, len: usize) {
        if len == 0 {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => (i + 1) % len,
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    /// Moves selection to the previous item (wraps around).
    fn select_prev(&mut self, len: usize) {
        if len == 0 {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    len - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }
}
