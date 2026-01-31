//! SerialTUI entry point.

mod error;
mod serial;
mod types;
mod ui;

use std::{fs, path::Path, sync::Arc};

use anyhow::Result;

use crate::{serial::serial_manager::SerialManager, ui::Ui};

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

const CONFIG_PATH: &str = "config/ports.toml";

/// Ensures the config file exists, creating it with defaults if missing.
fn ensure_config() -> &'static str {
    let config_path = Path::new(CONFIG_PATH);

    if !config_path.exists() {
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).ok();
        }
        fs::write(config_path, DEFAULT_CONFIG).ok();
        eprintln!("Created default config at {CONFIG_PATH}");
    }

    CONFIG_PATH
}

fn main() -> Result<()> {
    let config_path = ensure_config();

    let mut serial_manager = SerialManager::new();
    serial_manager
        .load_config(config_path)
        .unwrap_or_else(|e| eprintln!("{e}"));

    let mut ui = Ui::new(Arc::new(serial_manager));
    ui.run()?;

    Ok(())
}
