//! Serial port communication layer.
//!
//! Provides abstractions for opening, reading from, and writing to serial ports.
//! The main entry point is [`SerialManager`](serial_manager::SerialManager) which
//! manages multiple port connections and provides a pub/sub interface for data.

mod error;
pub mod port_connection;
mod port_handle;
pub mod port_info;
pub mod serial_manager;

pub use error::SerialError;
