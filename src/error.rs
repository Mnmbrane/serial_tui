#[derive(Debug)]
pub enum AppError {
    Io(std::io::Error),
    InvalidTomlSerialize(toml::ser::Error),
    InvalidTomlDeserialize(toml::de::Error),
    InvalidPortName(&'static str),
    InvalidFilePath(&'static str),
    ConfigPortInsert(&'static str),
    InvalidDataBits(&'static str),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use AppError::*;
        match self {
            Io(e) => write!(f, "IO error: {e}"),
            InvalidTomlDeserialize(e) => write!(f, "Toml Deserialize error: {e}"),
            InvalidTomlSerialize(e) => write!(f, "IO error: {e}"),
            InvalidPortName(e) => write!(f, "Invalid Port Name: {e}"),
            InvalidFilePath(e) => write!(f, "Invalid File Path: {e}"),
            ConfigPortInsert(e) => write!(f, "Could not insert new port element: {e}"),
            InvalidDataBits(e) => write!(f, "Invalid data bits: {e}"),
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
        AppError::InvalidTomlDeserialize(value)
    }
}

impl From<toml::ser::Error> for AppError {
    fn from(value: toml::ser::Error) -> Self {
        AppError::InvalidTomlSerialize(value)
    }
}
