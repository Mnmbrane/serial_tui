use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct PortConfig {
    pub path: PathBuf,
    pub baud_rate: u32,
    pub data_bits: u8,
    pub stop_bits: u8,
    pub parity: String,
    pub flow_control: String,
    pub line_ending: String,
    pub color: String,
}

impl Default for PortConfig {
    fn default() -> Self {
        Self {
            path: PathBuf::new(),
            baud_rate: 115200,
            data_bits: 8,
            stop_bits: 1,
            parity: "none".into(),
            flow_control: "none".into(),
            line_ending: "\n".into(),
            color: "green".into(),
        }
    }
}

// Unit tests
#[cfg(test)] // Only compiels the module during testing
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let port_config: PortConfig = PortConfig::default();
        assert_eq!(port_config.baud_rate, 115200);
        assert_eq!(port_config.data_bits, 8);
        assert_eq!(port_config.stop_bits, 1);
        assert_eq!(port_config.parity, "none".to_string());
        assert_eq!(port_config.flow_control, "none".to_string());
        assert_eq!(port_config.line_ending, "\n".to_string());
        assert_eq!(port_config.color, "green".to_string());
    }
    #[test]
    fn test_diff_baud_and_le() {
        let mut port_config: PortConfig = PortConfig::default();
        port_config.baud_rate = 9600;
        port_config.line_ending = "\r\n".to_string();
        assert_eq!(port_config.baud_rate, 9600);
        assert_eq!(port_config.line_ending, "\r\n");
    }
}
