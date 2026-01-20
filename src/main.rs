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

use tokio::sync::broadcast;

use crate::{
    error::AppError,
    serial::{port_connection::PortEvent, serial_manager::SerialManager},
};

fn main() -> Result<(), AppError> {
    // Create serial handler and give port mapping to it
    let mut serial_manager = SerialManager::new();
    serial_manager.from_file("config/ports.toml")?;

    // Create Logger

    // Create Notification System

    // Start UI(Ratatui)

    Ok(())
}
