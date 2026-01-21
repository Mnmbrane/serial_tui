//! UI widgets for the main layout areas.
//!
//! Each widget follows a common pattern:
//! - `render(&self, frame, area, focused)` - draws the widget
//! - `handle_key(&mut self, key) -> Option<Action>` - processes input, returns action
//!
//! The parent UI handles returned actions (e.g., opening popups, sending data).

mod config_bar;
mod display;
mod input_bar;

pub use config_bar::{ConfigAction, ConfigBar};
pub use display::{Display, DisplayAction};
pub use input_bar::{InputBar, InputBarAction};

use ratatui::{
    style::{Color, Style},
    widgets::{Block, Borders},
};

/// Creates a bordered block with focus indication.
///
/// When focused, the border is highlighted in yellow.
/// Used by all main widgets for consistent styling.
pub fn focused_block(title: &str, focused: bool) -> Block {
    let style = if focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(style)
}
