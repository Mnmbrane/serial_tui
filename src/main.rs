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

use crate::{error::AppError, serial::Serial, types::port_map::PortMap};

fn main() -> Result<(), AppError> {
    // Create the port mapping from the config
    let port_map = PortMap::new().from_file("config/ports.toml")?;

    // Create serial handler and give port mapping to it
    let serial_handler = Serial::new(port_map);

    // Create Notification System

    // Create Logger

    // Start UI(Ratatui)

    Ok(())
}
