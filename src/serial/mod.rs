//! Serial port communication layer.
//!
//! Provides abstractions for opening, reading from, and writing to serial ports.
//! The main entry point is [`SerialHub`](hub::SerialHub) which manages multiple
//! port connections and provides a pub/sub interface for data.

pub mod config;
pub mod connection;
mod error;
mod handle;
pub mod hub;

pub use error::SerialError;
