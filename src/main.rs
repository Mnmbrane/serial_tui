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
mod ui;

use std::{fs, path::Path, sync::Arc};

use crate::{error::AppError, serial::serial_manager::SerialManager, ui::Ui};

/// Default configuration created at runtime if no config exists
const DEFAULT_CONFIG: &str = r###"# SerialTUI Configuration
#
# Each section defines a serial port connection.
# The section name (e.g., [device1]) is used as the display name.
#
# Required:
#   path       - Device path (Linux: "/dev/ttyUSB0", Windows: "COM3")
#
# Optional:
#   baud_rate  = 115200    # Baud rate
#   line_ending = "lf"     # lf, cr, or crlf
#   color      = "white"   # Named color or hex "#RRGGBB"

# Example configuration:
# [device1]
# path = "/dev/ttyUSB0"  # Linux
# # path = "COM3"        # Windows
# baud_rate = 115200
# color = "green"

# [device2]
# path = "/dev/ttyUSB1"
# baud_rate = 9600
# color = "#FF5733"
"###;

/// Config file path
const CONFIG_PATH: &str = "config/ports.toml";

/// Ensures the config file exists, creating it with defaults if missing.
/// Returns the path to the config file.
fn ensure_config() -> &'static str {
    let config_path = Path::new(CONFIG_PATH);

    if !config_path.exists() {
        // Create config directory if needed
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).ok();
        }
        // Write default config
        fs::write(config_path, DEFAULT_CONFIG).ok();
        eprintln!("Created default config at {}", CONFIG_PATH);
    }

    CONFIG_PATH
}

fn main() -> Result<(), AppError> {
    // Ensure config exists (creates default if missing)
    let config_path = ensure_config();

    // Create serial handler and give port mapping to it
    let mut serial_manager = SerialManager::new();
    serial_manager
        .from_file(config_path)
        .unwrap_or_else(|e| eprintln!("{e}"));

    // Start UI(ratatui)
    let mut ui = Ui::new(Arc::new(serial_manager));
    ui.run()?;

    Ok(())
}
