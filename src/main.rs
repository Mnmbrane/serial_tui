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

fn main() -> Result<(), AppError> {
    let app_state = AppState::new().inspect_err(|e| eprintln!("{e}"))?;
    app_state.init().map_err(|e| eprintln!("{e}")).unwrap();
    Ok(())
}
