pub mod port_config;
pub use port_config::PortConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::AppError;

#[derive(Serialize, Deserialize)]
pub struct PortConfigMap(HashMap<String, PortConfig>);
// Want just 2 differnt configs for now.
// 1. PortConfig - Contains com port details
// 2. MacroConfig - Contains keybindings for VIM Motions (TODO)
#[derive(Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(flatten)]
    port_config: HashMap<String, PortConfig>,
}

impl AppConfig {
    pub fn new() -> Self {
        Self {
            port_config: HashMap::new(),
        }
    }

    pub fn init(&mut self, cfg: &str) -> Result<(), AppError> {
        self.port_config = toml::from_str(cfg)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {

    use crate::config::port_config::Color;

    use super::*;

    #[test]
    fn test_port_cfg() {
        let toml_str = r##"
            [USB0]
            path = "/dev/ttyUSB0"
            baud_rate = 115200
            data_bits = 8
            stop_bits = 1
            parity = "none"
            flow_control = "none"
            line_ending = "\n"
            color = "green"

            [USB1]
            path = "/dev/ttyUSB1"
            baud_rate = 9600

            [ACM0]
            path = "/dev/ttyACM0"
            baud_rate = 115200
            color = "#FF5500"
"##;
        let mut app_config: AppConfig = AppConfig::new();
        app_config.init(toml_str).unwrap();
        assert_eq!(app_config.port_config.len(), 3);
        assert_eq!(
            app_config.port_config.get("USB0").unwrap().baud_rate,
            Some(115200)
        );
        assert_eq!(
            app_config.port_config.get("ACM0").unwrap().color,
            Some(Color::Rgb {
                r: 255,
                g: 85,
                b: 0,
            })
        );
    }
}
