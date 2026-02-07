//! SerialTUI entry point.

mod app;
mod config;
mod error;
mod notify;
mod serial;
mod types;
mod ui;

use anyhow::Result;

use crate::app::App;

fn main() -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    let _guard = rt.enter();
    let mut app = App::new();
    app.run()?;
    Ok(())
}
