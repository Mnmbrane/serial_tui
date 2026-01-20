mod config_bar;
mod display;
mod input_bar;

pub use config_bar::{ConfigAction, ConfigBar};
pub use display::Display;
pub use input_bar::InputBar;

use ratatui::{
    style::{Color, Style},
    widgets::{Block, Borders},
};

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
