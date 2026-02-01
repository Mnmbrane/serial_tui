//! SerialTUI entry point.

mod config;
mod error;
mod serial;
mod types;
mod ui;

use std::sync::Arc;

use anyhow::Result;

use crate::{serial::hub::SerialHub, ui::Ui};

fn main() -> Result<()> {
    let config_path = config::ensure_config();

    let mut hub = SerialHub::new();
    hub.load_config(config_path)
        .unwrap_or_else(|e| eprintln!("{e}"));

    let mut ui = Ui::new(Arc::new(hub));
    ui.run()?;

    Ok(())
}
