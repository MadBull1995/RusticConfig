use crate::{error::ConfigError, ConfigMap, FilePath};

pub trait Reader {
    fn read(&self, path: &str) -> Result<ConfigMap, ConfigError>;
}

pub struct YamlConfigReader;

impl Reader for YamlConfigReader {
    fn read(&self, path: &str) -> Result<ConfigMap, ConfigError> {
        let file = std::fs::File::open(path)
            .map_err(|e| ConfigError::FileReadError(FilePath::new(path), e.to_string()))?;

        serde_yaml::from_reader(&file).map_err(|e| ConfigError::ParseError(e.to_string()))
    }
}

pub struct JsonConfigReader;

impl Reader for JsonConfigReader {
    fn read(&self, path: &str) -> Result<ConfigMap, ConfigError> {
        let file = std::fs::File::open(path)
            .map_err(|e| ConfigError::FileReadError(FilePath::new(path), e.to_string()))?;

        serde_json::from_reader(&file).map_err(|e| ConfigError::ParseError(e.to_string()))
    }
}
