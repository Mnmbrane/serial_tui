use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use super::focused_block;

pub enum ConfigAction {
    OpenPorts,
    AddPort,
}

pub struct ConfigBar {}

impl ConfigBar {
    pub fn new() -> Self {
        Self {}
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let block = focused_block(" Config ", focused);

        let content = Line::from(vec![
            Span::styled("[p]", Style::default().fg(Color::Yellow)),
            Span::styled("orts", Style::default().fg(Color::Cyan)),
            Span::raw("  "),
            Span::styled("[a]", Style::default().fg(Color::Yellow)),
            Span::styled("dd", Style::default().fg(Color::Green)),
        ]);

        let paragraph = Paragraph::new(content).block(block);
        frame.render_widget(paragraph, area);
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<ConfigAction> {
        match key.code {
            KeyCode::Char('p') => Some(ConfigAction::OpenPorts),
            KeyCode::Char('a') => Some(ConfigAction::AddPort),
            _ => None,
        }
    }
}
