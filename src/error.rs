#[derive(Debug)]
pub enum AppError {
    Io(std::io::Error),
    TomlDe(toml::de::Error),
    TomlSer(toml::ser::Error),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use AppError::*;
        match self {
            Io(e) => write!(f, "IO error: {e}"),
            TomlDe(e) => write!(f, "Toml Deserialize error: {e}"),
            TomlSer(e) => write!(f, "IO error: {e}"),
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        AppError::Io(value)
    }
}

impl From<toml::de::Error> for AppError {
    fn from(value: toml::de::Error) -> Self {
        AppError::TomlDe(value)
    }
}

impl From<toml::ser::Error> for AppError {
    fn from(value: toml::ser::Error) -> Self {
        AppError::TomlSer(value)
    }
}
