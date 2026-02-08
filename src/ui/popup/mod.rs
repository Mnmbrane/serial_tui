//! Popup system for modal dialogs.
//!
//! Popups are rendered on top of the main UI and capture all keyboard
//! input while visible. They don't store port data - it's passed in
//! during render/handle_key to stay in sync with the serial manager.

mod notification;
mod port_list;
mod send_group;

pub use notification::Notification;
pub use port_list::{PortListAction, PortListPopup};
pub use send_group::{SendGroupAction, SendGroupPopup};

use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout, Rect},
    widgets::{Clear, ListState},
};

/// Helper for creating centered popup overlays.
///
/// Calculates a centered rectangle based on percentage of screen size.
/// Used by `PortListPopup` and `SendGroupPopup`.
pub struct Popup {
    /// Width as percentage of screen (0-100)
    width_percent: u16,
    /// Height as percentage of screen (0-100)
    height_percent: u16,
}

impl Popup {
    /// Creates a new popup with the specified dimensions.
    ///
    /// Both values are percentages (e.g., 40 = 40% of screen).
    pub fn new(width_percent: u16, height_percent: u16) -> Self {
        Self {
            width_percent,
            height_percent,
        }
    }

    /// Calculates the centered rectangle for popup content.
    ///
    /// Uses flex centering to position the popup in the middle of the screen.
    pub fn area(&self, frame_area: Rect) -> Rect {
        let vertical =
            Layout::vertical([Constraint::Percentage(self.height_percent)]).flex(Flex::Center);
        let horizontal =
            Layout::horizontal([Constraint::Percentage(self.width_percent)]).flex(Flex::Center);

        let [area] = vertical.areas(frame_area);
        let [area] = horizontal.areas(area);
        area
    }

    /// Clears the popup area before rendering content.
    ///
    /// Call this before rendering popup widgets to erase what's behind it.
    pub fn clear(&self, frame: &mut Frame, area: Rect) {
        frame.render_widget(Clear, area);
    }
}

/// Moves a `ListState` selection to the next item (wraps around).
fn select_next(state: &mut ListState, len: usize) {
    if len == 0 {
        return;
    }
    let i = match state.selected() {
        Some(i) => (i + 1) % len,
        None => 0,
    };
    state.select(Some(i));
}

/// Moves a `ListState` selection to the previous item (wraps around).
fn select_prev(state: &mut ListState, len: usize) {
    if len == 0 {
        return;
    }
    let i = match state.selected() {
        Some(0) | None => len - 1,
        Some(i) => i - 1,
    };
    state.select(Some(i));
}
