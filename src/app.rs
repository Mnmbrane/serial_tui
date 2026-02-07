use std::sync::Arc;

use anyhow::Result;
use tokio::sync::mpsc;

use crate::{config, notify::Notify, serial::hub::SerialHub, ui::Ui};

pub struct App {
    hub: Arc<SerialHub>,
    ui: Ui,
}

impl App {
    /// Creates the hub, channels, and UI.
    pub fn new() -> Self {
        let config_path = config::ensure_config();

        let (notify_tx, notify_rx) = mpsc::unbounded_channel::<Notify>();

        let (mut hub, port_recv_rx) = SerialHub::new(notify_tx);
        hub.load_config(config_path)
            .unwrap_or_else(|e| eprintln!("{e}"));

        let hub = Arc::new(hub);
        let ui = Ui::new(hub.clone(), port_recv_rx, notify_rx);

        Self { hub, ui }
    }

    /// Runs the application.
    pub fn run(&mut self) -> Result<()> {
        self.ui.run()
    }
}
