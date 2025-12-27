use crate::config::ConfigError;

#[derive(Debug)]
pub enum AppError {
    ConfigError(ConfigError),
}

impl From<ConfigError> for AppError {
    fn from(e: ConfigError) -> Self {
        AppError::ConfigError(e)
    }
}
