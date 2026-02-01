//! Serial port communication layer.

pub mod connection;
mod error;
mod handle;
pub mod hub;

pub use error::SerialError;
