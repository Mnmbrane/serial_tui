use crossterm::event::KeyEvent;
use ratatui::{Frame, layout::Rect};

use super::focused_block;

pub struct Display {}

impl Display {
    pub fn new() -> Self {
        Self {}
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, focused: bool) {
        frame.render_widget(focused_block(" Display ", focused), area);
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            _ => {}
        }
    }
}
