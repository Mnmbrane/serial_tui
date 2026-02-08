//! SerialTUI entry point.

mod config;
mod error;
mod logger;
mod serial;
mod types;
mod ui;

use anyhow::Result;
use tokio::sync::mpsc;

use crate::{serial::hub::SerialHub, ui::Ui};

fn main() -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    let _guard = rt.enter();

    let config_path = config::ensure_config();

    // Create channels
    let (log_tx, log_rx) = mpsc::unbounded_channel();
    let (ui_tx, ui_rx) = mpsc::unbounded_channel();

    // Start serial hub
    let mut hub = SerialHub::new(ui_tx, log_tx);
    hub.load_config(config_path)
        .unwrap_or_else(|e| eprintln!("{e}"));

    // Start Logger
    tokio::spawn(logger::run(log_rx));

    // UI will own the serial hub
    let mut ui = Ui::new(hub, ui_rx);
    ui.run()?;

    Ok(())
}
