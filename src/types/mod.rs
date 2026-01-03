//! Core types used throughout the application.
//!
//! Contains shared data structures for serial port configuration,
//! terminal colors, and port management.

pub mod color;
pub mod port_info;
pub mod port_map;

pub use color::Color;
pub use port_info::PortInfo;
