//! # Rustic Config
//!
//! `rustic_config` is a Rust library designed for easy and efficient configuration management in Rust applications. It supports various sources like JSON, YAML files, environment variables, and command-line arguments. The library is built to be flexible, allowing easy access to different types of configuration values.
//!
//! ## Features
//!
//! - Load configurations from JSON and YAML files.
//! - Read configuration values from environment variables.
//! - Override configurations via command-line arguments.
//! - Support for custom data types through Serde.
//! - Easy to use API for accessing configuration values.
//!
//! ## Usage
//!
//! Add `rustic_config` to your `Cargo.toml`:
//! ```bash
//! cargo add rustic_config
//! ```
//!
//! Basic usage example:
//!
//! ```should_panic
//! use rustic_config::{ConfigManagerBuilder, ConfigSource, FilePath, error::ConfigError};
//! fn main() -> Result<(), ConfigError> {
//!     let mut builder = ConfigManagerBuilder::new();
//!     builder.add_source(ConfigSource::File(FilePath::new("config.yaml")));
//!     let config_manager = builder.build()?;
//!
//!     let db_url: String = config_manager.get_string("database_url").unwrap();
//!     println!("Database URL: {}", db_url);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Modules
//!
//! - `file_reader`: Provides functionality to read configurations from various sources.
//! - `env_vars`: Provides functionality to parse configurations from environment variables.
//! - `cli_flags`: Provides functionality to parse configurations from cli flags.
//! - `error`: Defines error types used throughout the library.

pub use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::HashMap,
    fmt::{self, Display},
    fs::File,
    io::Write,
    path::Path,
};
pub mod cli_flags;
pub mod env_vars;
pub mod error;
pub mod file_reader;
pub mod manager;
pub type ConfigMap = HashMap<String, Value>;

pub use manager::{ConfigManager, ConfigManagerBuilder};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FileType {
    Json,
    Yaml,
    Unsupported(FilePath),
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
pub struct FilePath(String);

impl FilePath {
    pub fn new<S: AsRef<str>>(name: S) -> Self {
        FilePath(name.as_ref().to_string())
    }

    pub fn file_type(&self) -> FileType {
        if self.0.ends_with(".yaml") {
            FileType::Yaml
        } else if self.0.ends_with(".json") {
            FileType::Json
        } else {
            FileType::Unsupported(self.clone())
        }
    }
}

impl AsRef<str> for FilePath {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for FilePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{:?}", self.0))
    }
}

impl Display for FilePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum ConfigSource {
    File(FilePath),
    Environment,
    CommandLine(Vec<String>),
}

/// Serializes a struct to a configuration file.
///
/// # Arguments
///
/// * `config` - The struct to serialize.
/// * `path` - File path where the configuration should be saved.
///
/// # Errors
///
/// Returns an error if file creation or serialization fails.
pub fn serialize_to_file<T>(config: &T, path: &Path) -> Result<(), std::io::Error>
where
    T: Serialize,
{
    let serialized = serde_json::to_string_pretty(config)?;
    let mut file = File::create(path)?;
    file.write_all(serialized.as_bytes())?;
    Ok(())
}

#[cfg(test)]
pub mod test {
    use crate::{manager::ConfigManagerBuilder, FilePath};
    use serde::Deserialize;
    use serde_json::Number;
    const TEST_YAML_PATH: &str = "test/test.yaml";
    const TEST_JSON_PATH: &str = "test/test.json";

    #[test]
    pub fn setup_yaml() {
        let mut cmb = ConfigManagerBuilder::new();
        cmb.add_source(crate::ConfigSource::File(FilePath::new(TEST_YAML_PATH)));
        let cm = cmb.build();
        assert_eq!(cm.is_ok(), true);
        if let cfg = cm.unwrap() {
            assert_eq!(cfg.get_i64("SOME_INT"), Some(1));
            assert_eq!(cfg.get_f64("SOME_FLOAT"), Some(1.1));
            assert_eq!(cfg.get_u64("SOME_UINT"), Some(999));
        }
    }

    #[test]
    pub fn setup_json() {
        let mut cmb = ConfigManagerBuilder::new();
        cmb.add_source(crate::ConfigSource::File(FilePath::new(TEST_JSON_PATH)));
        let cm = cmb.build();
        println!("{:?}", cm);
        assert_eq!(cm.is_ok(), true);
        if let cfg = cm.unwrap() {
            assert_eq!(cfg.get_i64("someInt"), Some(42));
            assert_eq!(cfg.get_f64("someFloat"), Some(42.1));
            assert_eq!(cfg.get_str("someString"), Some("Hello World!"));
        }
    }

    #[test]
    pub fn no_file_error() {
        let mut cmb = ConfigManagerBuilder::new();
        cmb.add_source(crate::ConfigSource::File(FilePath::new("no_file.yaml")));
        let cm = cmb.build();
        assert_eq!(cm.unwrap_err().to_string(), "Failed to read configuration file: No such file or directory (os error 2) [no_file.yaml]");
    }

    #[test]
    pub fn test_try() {
        let mut cmb = ConfigManagerBuilder::new();
        cmb.add_source(crate::ConfigSource::File(FilePath::new(TEST_YAML_PATH)));
        let cm = cmb.build();
        assert_eq!(cm.is_ok(), true);
        if let cfg = cm.unwrap() {
            assert_eq!(cfg.try_get("SOME_INT").unwrap(), 1);
        }
    }

    #[test]
    pub fn test_vec() {
        let mut cmb = ConfigManagerBuilder::new();
        cmb.add_source(crate::ConfigSource::File(FilePath::new(TEST_YAML_PATH)));
        let cm = cmb.build();
        assert_eq!(cm.is_ok(), true);
        if let cfg = cm.unwrap() {
            let new_vec: Vec<String> = cfg.get_vec("SOME_VEC").unwrap();
            assert_eq!(new_vec, vec!["1", "2", "3"]);
        }
    }

    #[test]
    pub fn test_struct() {
        #[derive(Debug, Deserialize)]
        struct MyConfig {
            TEST_KEY_INT: i64,
            TEST_KEY_FLOAT: f64,
            TEST_KEY_VEC: Vec<String>,
        }
        let mut cmb = ConfigManagerBuilder::new();
        cmb.add_source(crate::ConfigSource::File(FilePath::new(TEST_YAML_PATH)));
        let cm = cmb.build().unwrap();
        let my_cfg = cm.get_struct::<MyConfig>("SOME_OBJ").unwrap();
        assert_eq!(my_cfg.TEST_KEY_INT, 1);
        assert_eq!(my_cfg.TEST_KEY_FLOAT, 1.1);
        assert_eq!(my_cfg.TEST_KEY_VEC.len(), 3);
    }

    #[test]
    pub fn test_iter() {
        let mut cmb = ConfigManagerBuilder::new();
        cmb.add_source(crate::ConfigSource::File(FilePath::new(TEST_YAML_PATH)));
        let cm = cmb.build().unwrap();
        assert_eq!(cm.values().collect::<Vec<_>>().len(), 5);
    }

    // #[test]
    // pub fn test_file_watch() {
    //     let mut cmb = ConfigManagerBuilder::new();
    //     cmb.add_source(crate::ConfigSource::File(FilePath::new(TEST_YAML_PATH)));
    //     let cm = cmb.build().unwrap();
    //     let file_events = cm.watch_file_changes(FilePath("test/test.json".to_string())).unwrap();
    //     // This can be done in a separate thread or as part of an event loop
    //     for event in file_events {
    //         match event {
    //             Ok(event) => {
    //                 // Handle the file change event
    //                 println!("File changed: {:?}", event);
    //                 // Optionally, reload the configuration here
    //             },
    //             Err(e) => println!("Watch error: {:?}", e),
    //         }
    // }

    //
    // assert_eq!(cm.values().collect::<Vec<_>>().len(), 5);
    // }
}
