//! Serial port communication layer.

mod error;
pub mod hub;
pub mod port;

pub use error::SerialError;
pub use port::PortEvent;
