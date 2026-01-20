mod port_list;

pub use port_list::{PortEntry, PortListPopup};

use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout, Rect},
    widgets::Clear,
};

/// Renders a centered popup overlay
pub struct Popup {
    width_percent: u16,
    height_percent: u16,
}

impl Popup {
    pub fn new(width_percent: u16, height_percent: u16) -> Self {
        Self {
            width_percent,
            height_percent,
        }
    }

    /// Returns the centered area for the popup content
    pub fn area(&self, frame_area: Rect) -> Rect {
        let vertical =
            Layout::vertical([Constraint::Percentage(self.height_percent)]).flex(Flex::Center);
        let horizontal =
            Layout::horizontal([Constraint::Percentage(self.width_percent)]).flex(Flex::Center);

        let [area] = vertical.areas(frame_area);
        let [area] = horizontal.areas(area);
        area
    }

    /// Clears the popup area (call before rendering popup content)
    pub fn clear(&self, frame: &mut Frame, area: Rect) {
        frame.render_widget(Clear, area);
    }
}
