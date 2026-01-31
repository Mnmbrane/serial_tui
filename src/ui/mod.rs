//! Terminal user interface built with Ratatui.
//!
//! The UI consists of three main widgets (ConfigBar, Display, InputBar)
//! and a popup system for modal dialogs (port list, send group selection,
//! notifications).

mod popup;
mod ui;
mod widgets;

pub use popup::{PortListAction, PortListPopup, SendGroupAction, SendGroupPopup};
pub use ui::Ui;
