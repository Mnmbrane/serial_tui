//! SerialTUI entry point.
//!
//! Sets up the async runtime, creates channels between components,
//! spawns background tasks, and runs the UI loop.
//!
//! ## Tasks spawned
//! - Serial handler (manages port tasks)
//! - Display buffer updater (broadcast -> AppState)
//! - Logger (broadcast -> log files)
//! - Notification system (queue -> AppState)
//!
//! ## Shutdown
//! All tasks receive shutdown signal via channel close or AppState.running flag.

mod config;
mod error;
mod serialtui;

use error::AppError;
use serialtui::SerialTui;

fn main() -> Result<(), AppError> {
    SerialTui::new()?.setup_ports()?;
    Ok(())
}
