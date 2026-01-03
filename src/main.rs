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

mod error;
mod serial;
mod types;

use crate::{error::AppError, serial::Serial, types::port_map::PortMap};

fn main() -> Result<(), AppError> {
    // Create the port mapping from the config
    let port_map = PortMap::new().from_file("config/ports.toml")?;

    // Create serial handler and give port mapping to it
    let serial_handler = Serial::new(port_map);

    Ok(())
}
