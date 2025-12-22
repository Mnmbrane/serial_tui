use serde::{Deserialize, Serialize};

const fn default_baud_rate() -> u32 {
    115200
}

const fn default_data_bits() -> u8 {
    8
}

const fn default_stop_bits() -> u8 {
    1
}

fn default_parity() -> String {
    "none".to_string()
}

fn default_flow_control() -> String {
    "none".to_string()
}

fn default_line_ending() -> String {
    "\n".to_string()
}

fn default_color() -> String {
    "green".to_string()
}

#[derive(Serialize, Deserialize)]
pub struct PortConfig {
    name: String,
    path: String,

    #[serde(default = "default_baud_rate")]
    baud_rate: u32,
    #[serde(default = "default_data_bits")]
    data_bits: u8,
    #[serde(default = "default_stop_bits")]
    stop_bits: u8,
    #[serde(default = "default_parity")]
    parity: String,
    #[serde(default = "default_flow_control")]
    flow_control: String,
    #[serde(default = "default_line_ending")]
    line_ending: String,
    #[serde(default = "default_color")]
    color: String,
}

impl PortConfig {
    fn new(name: String, path: String) -> Self {
        Self {
            name,
            path,
            baud_rate: default_baud_rate(),
            data_bits: default_data_bits(),
            stop_bits: default_stop_bits(),
            parity: default_parity(),
            flow_control: default_flow_control(),
            line_ending: default_line_ending(),
            color: default_color(),
        }
    }
    fn baud_rate(mut self, baud_rate: u32) -> Self {
        self.baud_rate = baud_rate;
        self
    }

    fn data_bits(mut self, data_bits: u8) -> Self {
        self.data_bits = data_bits;
        self
    }

    fn stop_bits(mut self, stop_bits: u8) -> Self {
        self.stop_bits = stop_bits;
        self
    }

    fn parity(mut self, parity: String) -> Self {
        self.parity = parity;
        self
    }

    fn flow_control(mut self, flow_control: String) -> Self {
        self.flow_control = flow_control;
        self
    }

    fn line_ending(mut self, line_ending: String) -> Self {
        self.line_ending = line_ending;
        self
    }

    fn color(mut self, color: String) -> Self {
        self.color = color;
        self
    }
}

#[cfg(test)] // Only compiels the module during testing
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let port_confg: PortConfig = PortConfig::new("COM1".to_string(), "/dev/tty14".to_string());
        assert_eq!(port_confg.name, "COM1");
        assert_eq!(port_confg.path, "/dev/tty14");
        assert_eq!(port_confg.baud_rate, 115200);
        assert_eq!(port_confg.data_bits, 8);
        assert_eq!(port_confg.stop_bits, 1);
        assert_eq!(port_confg.parity, "none".to_string());
        assert_eq!(port_confg.flow_control, "none".to_string());
        assert_eq!(port_confg.line_ending, "\n".to_string());
        assert_eq!(port_confg.color, "green".to_string());
    }
    #[test]
    fn test_diff_baud_and_le() {
        let port_confg: PortConfig = PortConfig::new("COM2".to_string(), "/dev/tty15".to_string())
            .baud_rate(9600)
            .line_ending("\r\n".to_string());
        assert_eq!(port_confg.name, "COM2");
        assert_eq!(port_confg.path, "/dev/tty15");
        assert_eq!(port_confg.baud_rate, 9600);
        assert_eq!(port_confg.line_ending, "\r\n");
    }
}
