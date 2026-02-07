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

    let (notify_tx, notify_rx) = mpsc::unbounded_channel::<Notify>();
    let (log_tx, log_rx) = mpsc::unbounded_channel();

    let (mut hub, port_recv_rx) = SerialHub::new(notify_tx, log_tx);
    hub.load_config(config_path)
        .unwrap_or_else(|e| eprintln!("{e}"));

    tokio::spawn(logger::run(log_rx));

    let hub = Arc::new(hub);
    let mut ui = Ui::new(hub, port_recv_rx, notify_rx);
    ui.run()?;

    Ok(())
}
