//! Main display area for serial output.
//!
//! Will show received serial data with per-port color coding.
//! Currently a placeholder widget.

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{Frame, layout::Rect};

use crate::ui::{Notification, ui::Focus};

use super::focused_block;

/// Actions the display widget can request.
pub enum DisplayAction {
    /// Show a notification message
    Notify(String),
    /// Move focus to the input bar
    FocusInput,
}

/// Main area for displaying serial port output.
///
/// Will show scrollable output from all connected ports,
/// with each port's data in its configured color.
pub struct Display;

impl Display {
    /// Creates a new display widget.
    pub fn new() -> Self {
        Self {}
    }

    /// Renders the display area.
    ///
    /// TODO: Render serial output data here.
    pub fn render(&self, frame: &mut Frame, area: Rect, focused: bool) {
        frame.render_widget(focused_block(" Display ", focused), area);
    }

    /// Handles key input when this widget is focused.
    ///
    /// - `Enter` -> Move focus to input bar for typing
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<DisplayAction> {
        match key.code {
            KeyCode::Enter => Some(DisplayAction::FocusInput),
            _ => None,
        }
    }
}
