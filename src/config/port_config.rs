use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct PortConfig {
    path: PathBuf,
    baud_rate: u32,
    data_bits: u8,
    stop_bits: u8,
    parity: String,
    flow_control: String,
    line_ending: String,
    color: String,
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

impl PortConfig {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            ..Default::default()
        }
    }
    pub fn baud_rate(mut self, baud_rate: u32) -> Self {
        self.baud_rate = baud_rate;
        self
    }

    pub fn data_bits(mut self, data_bits: u8) -> Self {
        self.data_bits = data_bits;
        self
    }

    pub fn stop_bits(mut self, stop_bits: u8) -> Self {
        self.stop_bits = stop_bits;
        self
    }

    pub fn parity(mut self, parity: String) -> Self {
        self.parity = parity;
        self
    }

    pub fn flow_control(mut self, flow_control: String) -> Self {
        self.flow_control = flow_control;
        self
    }

    pub fn line_ending(mut self, line_ending: String) -> Self {
        self.line_ending = line_ending;
        self
    }

    pub fn color(mut self, color: String) -> Self {
        self.color = color;
        self
    }
}

// Unit tests
#[cfg(test)] // Only compiels the module during testing
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let port_config: PortConfig = PortConfig::new("/dev/tty14");
        assert_eq!(port_config.path, PathBuf::from("/dev/tty14"));
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
        let port_config: PortConfig = PortConfig::new("/dev/tty15")
            .baud_rate(9600)
            .line_ending("\r\n".to_string());
        assert_eq!(port_config.path, PathBuf::from("/dev/tty15"));
        assert_eq!(port_config.baud_rate, 9600);
        assert_eq!(port_config.line_ending, "\r\n");
    }
    #[test]
    fn test_builder() {
        let port_config: PortConfig = PortConfig::new("/dev/ttyACM0".to_string())
            .baud_rate(9600)
            .data_bits(7)
            .stop_bits(0)
            .parity("rand".to_string());
        assert_eq!(port_config.path, PathBuf::from("/dev/ttyACM0"));
        assert_eq!(port_config.baud_rate, 9600);
        assert_eq!(port_config.data_bits, 7);
        assert_eq!(port_config.stop_bits, 0);
        assert_eq!(port_config.parity, "rand");
    }
}
