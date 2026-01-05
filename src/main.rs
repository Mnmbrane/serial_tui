//! SerialTUI entry point.
//!
//! Sets up the async runtime, creates channels between components,
//! spawns background tasks, and runs the ratatui UI loop.
//!
//! ## Tasks spawned
//! - Serial handler (manages port I/O tasks)
//! - Ratatui UI (renders AppState to terminal)
//! - Logger (broadcast -> log files)
//! - Notification system (queue -> AppState)
//!
//! ## Shutdown
//! All tasks receive shutdown signal via channel close or AppState.running flag.

mod error;
mod serial;
mod types;

use crate::{error::AppError, serial::SerialManager};

fn main() -> Result<(), AppError> {
    // Create serial handler and give port mapping to it
    let SerialManager = SerialManager::from_file("config/ports.toml");

    // Create Notification System

    // Create Logger

    // Start UI(Ratatui)

    Ok(())
}
