//! Send group popup for selecting target ports.
//!
//! Allows the user to select which ports should receive typed input.
//! Uses checkboxes to show selection state.

use std::{collections::HashSet, sync::Arc};

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
};

use crate::config::PortConfig;

use super::Popup;

/// Actions returned by the send group popup.
pub enum SendGroupAction {
    /// Popup was closed (Esc pressed)
    Close,
}

/// Popup for selecting which ports to send data to.
///
/// Maintains a persistent set of selected port names that survives
/// popup close/reopen. Uses checkbox UI (`[x]` / `[ ]`).
pub struct SendGroupPopup {
    /// Helper for centered positioning
    popup: Popup,
    /// Current cursor position in the list
    list_state: ListState,
    /// Set of port names that are selected for sending
    selected: HashSet<String>,
    /// Whether the popup is currently shown
    pub visible: bool,
}

impl SendGroupPopup {
    /// Creates a new hidden send group popup.
    ///
    /// Uses 35% width, 50% height of the screen.
    pub fn new() -> Self {
        Self {
            popup: Popup::new(35, 50),
            list_state: ListState::default().with_selected(Some(0)),
            selected: HashSet::new(),
            visible: false,
        }
    }

    /// Toggles visibility, resetting cursor on open.
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
        if self.visible {
            self.list_state.select(Some(0));
        }
    }

    /// Opens the popup and moves cursor to first item.
    #[allow(dead_code)]
    pub fn open(&mut self) {
        self.visible = true;
        self.list_state.select(Some(0));
    }

    /// Closes the popup (selection is preserved).
    #[allow(dead_code)]
    pub fn close(&mut self) {
        self.visible = false;
    }

    /// Returns the currently selected port names.
    ///
    /// Used by the input bar to know where to send data.
    pub fn get_selected(&self) -> Vec<String> {
        self.selected.iter().cloned().collect()
    }

    /// Returns true if the port is in the selection set.
    #[allow(dead_code)]
    pub fn is_selected(&self, name: &str) -> bool {
        self.selected.contains(name)
    }

    /// Renders the checkbox list of ports.
    ///
    /// Each item shows: `[x] port_name  baud_rate` or `[ ] port_name  baud_rate`
    pub fn render(&mut self, frame: &mut Frame, ports: &[(String, Arc<PortConfig>)]) {
        if !self.visible {
            return;
        }

        let area = self.popup.area(frame.area());
        self.popup.clear(frame, area);

        // Build list items with checkbox state
        let items: Vec<ListItem> = ports
            .iter()
            .map(|(name, info)| {
                let checkbox = if self.selected.contains(name) {
                    "[x]"
                } else {
                    "[ ]"
                };

                let line = Line::from(vec![
                    Span::styled(format!("{checkbox} "), Style::default().fg(Color::Yellow)),
                    Span::raw(format!("{}  {}", name, info.baud_rate)),
                ]);
                ListItem::new(line)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title(" Send To ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::White)),
            )
            .highlight_style(Style::default().bg(Color::DarkGray));

        frame.render_stateful_widget(list, area, &mut self.list_state);
    }

    /// Handles key input when this popup is visible.
    ///
    /// - `Esc` -> Close popup
    /// - `Up/k` -> Move cursor up
    /// - `Down/j` -> Move cursor down
    /// - `Space/Enter` -> Toggle current port selection
    pub fn handle_key(
        &mut self,
        key: KeyEvent,
        ports: &[(String, Arc<PortConfig>)],
    ) -> Option<SendGroupAction> {
        match key.code {
            KeyCode::Esc => {
                self.visible = false;
                Some(SendGroupAction::Close)
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.select_prev(ports.len());
                None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.select_next(ports.len());
                None
            }
            KeyCode::Char(' ') | KeyCode::Enter => {
                self.toggle_selected(ports);
                None
            }
            _ => None,
        }
    }

    /// Toggles the selected state of the currently highlighted port.
    fn toggle_selected(&mut self, ports: &[(String, Arc<PortConfig>)]) {
        if let Some(i) = self.list_state.selected() {
            if let Some((name, _)) = ports.get(i) {
                if self.selected.contains(name) {
                    self.selected.remove(name);
                } else {
                    self.selected.insert(name.clone());
                }
            }
        }
    }

    /// Moves cursor to the next item (wraps around).
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

    /// Moves cursor to the previous item (wraps around).
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
