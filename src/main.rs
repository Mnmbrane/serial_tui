//! SerialTUI entry point.

mod config;
mod error;
mod logger;
mod notify;
mod serial;
mod types;
mod ui;

use std::sync::Arc;

use anyhow::Result;
use tokio::sync::mpsc;

use crate::{notify::Notify, serial::hub::SerialHub, ui::Ui};

fn main() -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    let _guard = rt.enter();

    let config_path = config::ensure_config();

    // Create channels
    let (notify_tx, notify_rx) = mpsc::unbounded_channel::<Notify>();
    let (log_tx, log_rx) = mpsc::unbounded_channel();
    let (port_recv_chan_tx, port_recv_chan_rx) = mpsc::unbounded_channel();

    // Start serial hub
    let mut hub = SerialHub::new(port_recv_chan_tx, notify_tx, log_tx);
    hub.load_config(config_path)
        .unwrap_or_else(|e| eprintln!("{e}"));

    // Start Logger
    tokio::spawn(logger::run(log_rx));

    // UI will own the serial hub
    let mut ui = Ui::new(hub, port_recv_chan_rx, notify_rx);
    ui.run()?;

    Ok(())
}
