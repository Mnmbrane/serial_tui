//! Terminal user interface built with Ratatui.
//!
//! The UI consists of three main widgets (ConfigBar, Display, InputBar)
//! and a popup system for modal dialogs (port list, send group selection,
//! notifications).

use std::sync::Arc;

use crate::serial::PortEvent;

mod popup;
mod ui;
mod widgets;

pub use popup::{PortListAction, PortListPopup, SendGroupAction, SendGroupPopup};
pub use ui::Ui;

/// Events sent to the UI from background components.
pub enum UiEvent {
    PortData(Arc<PortEvent>),
    ShowNotification(Arc<str>),
}
