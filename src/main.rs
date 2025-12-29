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

mod app_state;
mod config;
mod error;

use app_state::AppState;
use error::AppError;

const PORT_CONFIG_PATH: &'static str = "config/ports.toml";

fn main() -> Result<(), AppError> {
    let app_state = AppState::new().inspect_err(|e| eprintln!("{e}"))?;
    let _ = app_state
        .configure(PORT_CONFIG_PATH)
        .map_err(|e| eprintln!("{e}"));

    // Read the config

    // Start the serial readers and writers

    // Start logger

    // Start the UI

    Ok(())
}
