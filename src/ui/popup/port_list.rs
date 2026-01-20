use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
};

use super::Popup;

/// Entry for a port in the list
pub struct PortEntry {
    pub name: String,
    pub baud_rate: u32,
    pub connected: bool,
}

/// Popup showing list of COM ports with status
pub struct PortListPopup {
    popup: Popup,
    ports: Vec<PortEntry>,
    list_state: ListState,
    pub visible: bool,
}

impl PortListPopup {
    pub fn new() -> Self {
        Self {
            popup: Popup::new(40, 50),
            ports: Vec::new(),
            list_state: ListState::default(),
            visible: false,
        }
    }

    pub fn set_ports(&mut self, ports: Vec<PortEntry>) {
        self.ports = ports;
        if !self.ports.is_empty() {
            self.list_state.select(Some(0));
        }
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    pub fn render(&mut self, frame: &mut Frame) {
        if !self.visible {
            return;
        }

        let area = self.popup.area(frame.area());
        self.popup.clear(frame, area);

        let items: Vec<ListItem> = self
            .ports
            .iter()
            .map(|port| {
                let dot = if port.connected { "●" } else { "○" };
                let dot_color = if port.connected {
                    Color::Green
                } else {
                    Color::Red
                };

                let line = Line::from(vec![
                    Span::styled(format!("{} ", dot), Style::default().fg(dot_color)),
                    Span::raw(format!("{}  {}", port.name, port.baud_rate)),
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

    pub fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => self.visible = false,
            KeyCode::Up | KeyCode::Char('k') => self.select_prev(),
            KeyCode::Down | KeyCode::Char('j') => self.select_next(),
            KeyCode::Enter => {
                // TODO: handle port selection
            }
            _ => {}
        }
    }

    fn select_next(&mut self) {
        if self.ports.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => (i + 1) % self.ports.len(),
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn select_prev(&mut self) {
        if self.ports.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.ports.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }
}
