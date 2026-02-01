//! Configuration loading and management.

pub mod port;

use std::{fs, path::Path};

pub use port::PortConfig;

const CONFIG_PATH: &str = "config/ports.toml";

const DEFAULT_CONFIG: &str = r##"# SerialTUI Configuration
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
"##;

/// Ensures the config file exists, creating it with defaults if missing.
pub fn ensure_config() -> &'static str {
    let path = Path::new(CONFIG_PATH);

    if !path.exists() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).ok();
        }
        fs::write(path, DEFAULT_CONFIG).ok();
        eprintln!("Created default config at {CONFIG_PATH}");
    }

    CONFIG_PATH
}
