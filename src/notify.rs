//! Structured notification types for cross-component messaging.

use std::sync::Arc;

pub enum NotifyLevel {
    Info,
    Warn,
    Error,
}

pub struct Notify {
    pub level: NotifyLevel,
    pub source: Arc<str>,
    pub message: String,
}
