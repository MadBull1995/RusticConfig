use thiserror::Error;

use crate::{FilePath, FileType};

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ConfigError {
    #[error("Feature not passed {:?}", 0)]
    FeatureNotSupported(FileType),

    #[error("Must pass sources to read from")]
    EmptySources,

    #[error("Failed to read configuration file: {1} [{0}]")]
    FileReadError(FilePath, String),

    #[error("Error parsing configuration: {0}")]
    ParseError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Configuration profile not found: {0}")]
    ProfileNotFoundError(String),

    #[error("Value: {0} not found in configs map")]
    NullValue(String),

    #[error("The key {0} not found on configurations")]
    KeyNotFoundError(String),

    #[error("Failed to watch configuration file: {0}")]
    FileWatchError(String),
}
