use tokio::sync::mpsc;

use crate::{
    serial::{PortEvent, hub::SerialHub},
    ui::Ui,
};

pub struct App {
    hub: SerialHub,
    rx: mpsc::Receiver<PortEvent>,
    ui: Ui,
}

impl App {
    pub fn new() -> Self {
        //let (hub, rx) = SerialHub::new();
        todo!()
    }
}
