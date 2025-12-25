#[derive(Debug)]
pub enum ConfigError {
    Io(std::io::Error),
}

impl From<std::io::Error> for ConfigError {
    fn from(value: std::io::Error) -> Self {
        ConfigError::Io(value)
    }
}
