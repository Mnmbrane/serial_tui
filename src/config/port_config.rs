//! Uses the builder pattern
pub struct PortConfig {
    name: String,
    path: String,
    baud_rate: Option<u32>,
    data_bits: Option<u8>,
    stop_bits: Option<u8>,
    parity: Option<String>,
    flow_control: Option<String>,
    line_ending: Option<String>,
    color: Option<String>,
}

impl PortConfig {
    fn new(name: String, path: String) -> Self {
        Self {
            name,
            path,
            baud_rate: Some(115200),
            data_bits: Some(8),
            stop_bits: Some(1),
            parity: Some("none".to_string()),
            flow_control: Some("none".to_string()),
            line_ending: Some("\n".to_string()),
            color: Some("green".to_string()),
        }
    }

    fn baud_rate(mut self, baud_rate: u32) -> Self {
        self.baud_rate = Some(baud_rate);
        self
    }

    fn data_bits(mut self, data_bits: u32) -> Self {
        self.baud_rate = Some(data_bits);
        self
    }

    fn stop_bits(mut self, stop_bits: u32) -> Self {
        self.baud_rate = Some(stop_bits);
        self
    }

    fn parity(mut self, parity: u32) -> Self {
        self.baud_rate = Some(parity);
        self
    }

    fn flow_control(mut self, flow_control: u32) -> Self {
        self.baud_rate = Some(flow_control);
        self
    }

    fn line_ending(mut self, line_ending: u32) -> Self {
        self.baud_rate = Some(line_ending);
        self
    }

    fn color(mut self, color: u32) -> Self {
        self.baud_rate = Some(color);
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
        assert_eq!(port_confg.baud_rate, Some(115200));
        assert_eq!(port_confg.data_bits, Some(8));
        assert_eq!(port_confg.stop_bits, Some(1));
        assert_eq!(port_confg.parity, Some("none".to_string()));
        assert_eq!(port_confg.flow_control, Some("none".to_string()));
        assert_eq!(port_confg.line_ending, Some("\n".to_string()));
        assert_eq!(port_confg.color, Some("green".to_string()));
    }
    #[test]
    fn test_default_with_diff_baud() {
        let port_confg: PortConfig =
            PortConfig::new("COM1".to_string(), "/dev/tty14".to_string()).baud_rate(9600);
        assert_eq!(port_confg.name, "COM1");
        assert_eq!(port_confg.path, "/dev/tty14");
        assert_eq!(port_confg.baud_rate, Some(9600));
    }
}
