#[cfg(feature = "watch")]
use notify::{Event, RecommendedWatcher, RecursiveMode, Result as NotifyResult, Watcher};
use std::{
    collections::{hash_map, HashMap},
    sync::mpsc::{channel, Receiver},
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{Map, Number, Value};
use crate::file_reader::{JsonConfigReader, Reader, YamlConfigReader};
use crate::{error::ConfigError, ConfigMap, ConfigSource, FilePath, FileType};

/// ConfigManagerBuilder is responsible for building the ConfigManager.
/// It allows adding various configuration sources like environment variables, files, and command-line arguments.
///
/// # Examples
///
/// ```should_panic
/// use rustic_config::{ConfigManagerBuilder, ConfigSource, FilePath};
///
/// let mut builder = ConfigManagerBuilder::new();
/// builder.add_source(ConfigSource::File(FilePath::new("config.yaml")));
/// let config_manager = builder.build().unwrap();
/// ```
pub struct ConfigManagerBuilder {
    sources: HashMap<ConfigSource, ConfigMap>,
}

impl Default for ConfigManagerBuilder {
    fn default() -> Self {
        let mut cmb = ConfigManagerBuilder::new();
        cmb.add_source(ConfigSource::Environment);
        cmb.add_source(ConfigSource::File(FilePath::new("config.yaml")));
        cmb
    }
}

/// A configuration manager builder, useful utility methods for crafting a [`ConfigManager`]
impl ConfigManagerBuilder {
    /// Creates a new ConfigManagerBuilder with no sources added.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustic_config::ConfigManagerBuilder;
    ///
    /// let builder = ConfigManagerBuilder::new();
    /// ```
    pub fn new() -> Self {
        Self {
            sources: HashMap::new(),
        }
    }

    // internal function to load sources
    fn load_sources(self) -> Result<ConfigMap, ConfigError> {
        let mut cfg_map = HashMap::new();

        for (_, mut src) in self.sources.into_iter().enumerate() {
            match src.0 {
                ConfigSource::File(path) => match path.file_type() {
                    FileType::Json => {
                        let reader = JsonConfigReader;
                        src.1 = reader.read(path.to_string().as_str())?;
                        for (k, v) in src.1.into_iter() {
                            cfg_map.insert(k, v);
                        }
                    }
                    FileType::Yaml => {
                        let reader = YamlConfigReader;
                        src.1 = reader.read(&path.to_string())?;
                        for (k, v) in src.1.into_iter() {
                            cfg_map.insert(k, v);
                        }
                    }
                    FileType::Unsupported(path) => {
                        return Err(ConfigError::FileReadError(path, "Unsupported".to_string()))
                    }
                },
                _ => unimplemented!(),
            }
        }

        Ok(cfg_map)
    }

    /// Add a new source of configuration to [`ConfigManager`]
    ///
    /// # Arguments
    ///
    /// * `src` - Configuration source to be added
    ///
    /// # Examples
    ///
    /// ```
    /// use rustic_config::{ConfigManagerBuilder, ConfigSource, FilePath};
    ///
    /// let mut builder = ConfigManagerBuilder::new();
    /// builder.add_source(ConfigSource::File(FilePath::new("config.json")));
    /// ```
    pub fn add_source(&mut self, src: ConfigSource) -> &mut Self {
        self.sources.insert(src, HashMap::new());
        self
    }

    /// Builds and returns the [`ConfigManager`] based on the added sources.
    ///
    /// # Errors
    ///
    /// Returns [`Err`] if no sources have been added or if there's an issue loading the configuration.
    ///
    /// # Examples
    ///
    /// ```should_panic
    /// use rustic_config::{ConfigManagerBuilder, ConfigSource, FilePath};
    ///
    /// let mut builder = ConfigManagerBuilder::new();
    /// builder.add_source(ConfigSource::File(FilePath::new("config.json")));
    /// let config_manager = builder.build().unwrap();
    /// ```
    pub fn build(self) -> Result<ConfigManager, ConfigError> {
        if self.sources.is_empty() {
            return Err(ConfigError::EmptySources);
        }
        let srcs = self.sources.keys().cloned().collect::<Vec<ConfigSource>>();
        let cfgs = self.load_sources()?;
        // let new_srcs: Vec<ConfigSource> = srcs.iter().cloned().collect();
        Ok(ConfigManager::new(cfgs, srcs))
    }
}

/// ConfigManager holds and manages the application's configuration.
///
/// # Examples
///
/// ```should_panic
/// use rustic_config::{ConfigManagerBuilder, ConfigSource, FilePath};
///
/// let mut builder = ConfigManagerBuilder::new();
/// builder.add_source(ConfigSource::File(FilePath::new("config.yaml")));
/// let config_manager = builder.build().unwrap();
///
/// let config_value = config_manager.get_string("my_config_key");
/// ```
#[derive(Debug)]
pub struct ConfigManager {
    configs: ConfigMap,
    sources: Vec<ConfigSource>,
}

impl ConfigManager {
    /// Creates a new ConfigManager with the given configuration map.
    ///
    /// # Arguments
    ///
    /// * `configs` - A map holding the configuration key-value pairs.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustic_config::{ConfigManager, ConfigMap};
    ///
    /// let configs = ConfigMap::new();
    /// let config_manager = ConfigManager::new(configs);
    /// ```
    pub fn new(configs: ConfigMap, sources: Vec<ConfigSource>) -> Self {
        Self { configs, sources }
    }

    /// Fetches a [`String`] value from the configuration.
    ///
    /// # Arguments
    ///
    /// * `key` - The key for the configuration value.
    ///
    /// # Returns
    ///
    /// Returns [`Some(String)`] if the key exists and the value is a string; otherwise [`None`].
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // assuming a configuration with a key "site_name" having value "MySite"
    /// let site_name = config_manager.get_string("site_name").unwrap();
    /// assert_eq!(site_name, "MySite");
    /// ```
    pub fn get_string(&self, key: &str) -> Option<String> {
        self.configs
            .get(key)
            .and_then(|v| v.as_str().map(String::from))
    }

    /// Fetches a string slice [`&str`] value from the configuration.
    ///
    /// # Arguments
    ///
    /// * `key` - The key for the configuration value.
    ///
    /// # Returns
    ///
    /// Returns [`Some(&str)`] if the key exists and the value is a string slice; otherwise [`None`].
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // assuming a configuration with a key "api_endpoint" having value "http://example.com/api"
    /// let api_endpoint = config_manager.get_str("api_endpoint").unwrap();
    /// assert_eq!(api_endpoint, "http://example.com/api");
    /// ```
    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.configs.get(key).and_then(|v| v.as_str())
    }

    /// Fetches a boolean ([`bool`]) value from the configuration.
    ///
    /// # Arguments
    ///
    /// * `key` - The key for the configuration value.
    ///
    /// # Returns
    ///
    /// Returns `Some(bool)` if the key exists and the value is a boolean; otherwise `None`.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // assuming a configuration with a key "feature_enabled" having value true
    /// let feature_enabled = config_manager.get_bool("feature_enabled").unwrap();
    /// assert!(feature_enabled);
    /// ```
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.configs.get(key).and_then(|v| v.as_bool())
    }

    /// Fetches an [`i64`] value from the configuration.
    ///
    /// # Arguments
    ///
    /// * `key` - The key for the configuration value.
    ///
    /// # Returns
    ///
    /// Returns [`Some(i64)`] if the key exists and the value is an integer; otherwise [`None`].
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // assuming a configuration with a key "max_connections" having value 100
    /// let max_connections = config_manager.get_i64("max_connections").unwrap();
    /// assert_eq!(max_connections, 100);
    /// ```
    pub fn get_i64(&self, key: &str) -> Option<i64> {
        self.configs.get(key).and_then(|v| v.as_i64())
    }

    /// Fetches an [`f64`] value from the configuration.
    ///
    /// # Arguments
    ///
    /// * `key` - The key for the configuration value.
    ///
    /// # Returns
    ///
    /// Returns [`Some(f64)`] if the key exists and the value is a floating-point number; otherwise [`None`].
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // assuming a configuration with a key "discount_rate" having value 0.15
    /// let discount_rate = config_manager.get_f64("discount_rate").unwrap();
    /// assert_eq!(discount_rate, 0.15);
    /// ```
    pub fn get_f64(&self, key: &str) -> Option<f64> {
        self.configs.get(key).and_then(|v| v.as_f64())
    }

    /// Fetches a [`u64`] value from the configuration.
    ///
    /// # Arguments
    ///
    /// * `key` - The key for the configuration value.
    ///
    /// # Returns
    ///
    /// Returns [`Some(u64)`] if the key exists and the value is an unsigned integer; otherwise [`None`].
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // assuming a configuration with a key "user_count" having value 5000
    /// let user_count = config_manager.get_u64("user_count").unwrap();
    /// assert_eq!(user_count, 5000);
    /// ```
    pub fn get_u64(&self, key: &str) -> Option<u64> {
        self.configs.get(key).and_then(|v| v.as_u64())
    }

    /// Fetches a [`serde_json::value::Number`] value from the configuration.
    ///
    /// # Arguments
    ///
    /// * `key` - The key for the configuration value.
    ///
    /// # Returns
    ///
    /// Returns [`Some(&Number)`] if the key exists and the value is a number; otherwise [`None`].
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // assuming a configuration with a key "pi" having value 3.14159
    /// let pi = config_manager.get_number("pi").unwrap();
    /// assert_eq!(pi, &serde_json::value::Number::from_f64(3.14159).unwrap());
    /// ```
    pub fn get_number(&self, key: &str) -> Option<&Number> {
        self.configs.get(key).and_then(|v| v.as_number())
    }

    /// Fetches a vector of `T` values from the configuration.
    ///
    /// # Type Parameters
    ///
    /// `T`: The type of the elements in the returned vector. Must be deserializable from a JSON value.
    ///
    /// # Arguments
    ///
    /// * `key` - The key for the configuration value.
    ///
    /// # Returns
    ///
    /// Returns [`Some(Vec<T>)`] if the key exists and the value is an array; otherwise [`None`].
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // assuming a configuration with a key "ids" having an array of integers
    /// let ids = config_manager.get_vec::<i32>("ids").unwrap();
    /// assert_eq!(ids, vec![1, 2, 3]);
    /// ```
    pub fn get_vec<T>(&self, key: &str) -> Option<Vec<T>>
    where
        T: DeserializeOwned + Serialize + Send + Sync + 'static,
    {
        self.configs.get(key).and_then(|v| {
            v.as_array()
                .unwrap()
                .into_iter()
                .map(|item| serde_json::from_value(item.clone()).ok())
                .collect::<Option<Vec<T>>>()
        })
    }

    /// Attempts to fetch a value from the configuration.
    ///
    /// # Arguments
    ///
    /// * `key` - The key for the configuration value.
    ///
    /// # Returns
    ///
    /// Returns [`Ok(&Value)`] if the key exists; otherwise [`Err(ConfigError::NullValue(key.to_owned()))`].
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // assuming a configuration with a key "timeout" having value 30
    /// let timeout = config_manager.try_get("timeout").unwrap();
    /// assert_eq!(*timeout, 30.into());
    /// ```
    pub fn try_get(&self, key: &str) -> Result<&Value, ConfigError> {
        if let Some(cfg) = self.configs.get(key) {
            Ok(cfg)
        } else {
            return Err(ConfigError::NullValue(key.to_owned()));
        }
    }

    /// Fetches the associated Map if the value is a JSON Object.
    ///
    /// # Arguments
    ///
    /// * `key` - The key for the configuration value.
    ///
    /// # Returns
    ///
    /// Returns `Some(&Map<String, Value>)` if the value is an object; otherwise `None`.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // assuming a configuration with a key "database" having an object with connection details
    /// let database_config = config_manager.get_object("database").unwrap();
    /// assert!(database_config.contains_key("username"));
    /// assert!(database_config.contains_key("password"));
    /// ```
    pub fn get_object(&self, key: &str) -> Option<&Map<String, Value>> {
        if let Some(v) = self.configs.get(key) {
            return v.as_object();
        } else {
            return None;
        }
    }

    /// Returns a mutable reference to the value corresponding to the key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key for the configuration value.
    ///
    /// # Returns
    ///
    /// Returns `Some(&mut Value)` if the key exists; otherwise `None`.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // assuming a configuration with a key "counter" initially set to 0
    /// if let Some(counter_val) = config_manager.get_mut("counter") {
    ///     *counter_val = serde_json::Value::from(1);
    /// }
    /// assert_eq!(config_manager.get_i64("counter").unwrap(), 1);
    /// ```
    pub fn get_mut(&mut self, key: &str) -> Option<&mut Value> {
        self.configs.get_mut(key)
    }

    /// Returns the key-value pair corresponding to the supplied key.
    pub fn get_key_value(&mut self, key: &str) -> Option<(&String, &Value)> {
        self.configs.get_key_value(key)
    }

    /// An iterator visiting all values in arbitrary order. The iterator element type is &'a V.
    pub fn values(&self) -> hash_map::Values<'_, String, Value> {
        self.configs.values()
    }

    /// Returns a deserialize struct
    ///
    /// # Examples
    /// ```should_panic
    /// use serde::Deserialize;
    /// use rustic_config::{ConfigManagerBuilder, ConfigSource, FilePath};
    ///
    ///  #[derive(Debug, Deserialize)]
    /// struct MyConfig {
    ///     TEST_KEY_INT: i64,
    ///     TEST_KEY_FLOAT: f64,
    ///     TEST_KEY_VEC: Vec<String>,
    /// }
    ///
    /// let mut cmb = ConfigManagerBuilder::new();
    /// cmb.add_source(ConfigSource::File(FilePath::new("config.yaml")));
    /// let cm = cmb.build().unwrap();
    ///
    /// let my_cfg = cm.get_struct::<MyConfig>("SOME_OBJ").unwrap();
    /// assert_eq!(my_cfg.TEST_KEY_INT, 1);
    /// ```
    pub fn get_struct<T>(&self, key: &str) -> Result<T, ConfigError>
    where
        T: DeserializeOwned,
    {
        self.configs
            .get(key)
            .ok_or_else(|| ConfigError::KeyNotFoundError(key.to_owned())) // Create this error variant if it doesn't exist
            .and_then(|v| {
                serde_json::from_value(v.clone())
                    .map_err(|e| ConfigError::ParseError(e.to_string()))
            })
    }

    /// Attempts to parse the entire configuration into a specified type `T`.
    ///
    /// # Type Parameters
    ///
    /// `T`: The type into which the configuration should be parsed. Must implement `DeserializeOwned`.
    ///
    /// # Returns
    ///
    /// Returns [`Ok(T)`] on successful parsing, or [`Err(ConfigError)`] on failure.
    ///
    /// # Examples
    ///
    /// Assuming `configs` contains JSON representing a `MyConfig` struct:
    ///
    /// ```ignore
    /// let my_config: MyConfig = config_manager.parse().unwrap();
    /// ```
    pub fn parse<T>(&self) -> Result<T, ConfigError>
    where
        T: DeserializeOwned,
    {
        let val = Self::convert_hashmap_to_value(self.configs.clone());
        serde_json::from_value(val).map_err(|e| ConfigError::ParseError(e.to_string()))
    }

    /// Removes a value from the configuration, leaving a Null in its place.
    ///
    /// # Arguments
    ///
    /// * `key` - The key for the configuration value to remove.
    ///
    /// # Returns
    ///
    /// Returns the removed `Value` if the key exists; otherwise, it leaves a Null in its place.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // assuming a configuration with a key "temporary_key"
    /// let removed_value = config_manager.take("temporary_key");
    /// assert_eq!(removed_value, serde_json::Value::Null);
    /// ```
    pub fn take(&mut self, key: &str) -> Value {
        self.configs.get_mut(key).unwrap().take()
    }

    #[cfg(feature = "watch")]
    /// Watch for configuration file changes
    ///
    /// # Examples
    ///
    /// ```should_panic
    /// use std::{thread, time::Duration};
    /// use rustic_config::{ConfigManagerBuilder, FilePath, ConfigSource, error::ConfigError};
    ///
    /// let mut config_manager_builder = ConfigManagerBuilder::new();
    /// config_manager_builder
    ///     .add_source(ConfigSource::File(FilePath::new("../test/test.json")));
    ///
    /// let config_manager = config_manager_builder
    ///     .build()?;
    ///
    /// let (term_tx, term_rx) = oneshot::channel();
    /// let rx = config_manager.watch_file_changes(term_rx)?;
    ///
    /// thread::spawn(move || {
    ///     for event in rx {
    ///         println!("File Watcher Event: {:?}", event.unwrap())
    ///     }
    /// });
    ///
    /// // Wait for 10 sec
    /// thread::sleep(Duration::from_secs(10));
    /// // When you want to stop watching the file
    /// term_tx.send(()).unwrap();
    ///
    /// # Ok::<(), ConfigError>(())
    /// ```
    pub fn watch_file_changes(
        &self,
        term_rx: oneshot::Receiver<()>,
    ) -> Result<Receiver<NotifyResult<Event>>, ConfigError> {
        use std::{path::Path, thread};

        let (tx, rx) = channel();
        let sources = self.sources.clone();

        // Create a watcher object in a separate thread
        thread::spawn(move || {
            let mut watcher: RecommendedWatcher = notify::recommended_watcher(move |res| {
                tx.send(res)
                    .expect("Failed to send file change notification");
            })
            .map_err(|e| ConfigError::FileWatchError(e.to_string()))
            .unwrap();

            if let Some(src) = sources.first() {
                match src {
                    ConfigSource::File(fp) => {
                        println!("watching file: {}", fp);

                        watcher
                            .watch(Path::new(&fp.as_ref()), RecursiveMode::Recursive)
                            .map_err(|e| ConfigError::FileWatchError(e.to_string()))
                            .unwrap();
                    }
                    ConfigSource::Environment | ConfigSource::CommandLine(_) => unreachable!(),
                }
            }
            // Block this thread until the shutdown signal is received
            match term_rx.recv() {
                Err(e) => println!("{:?}", e),
                Ok(_) => {
                    println!("File watcher shutting down.");
                }
            }
        });

        Ok(rx)
    }

    fn convert_hashmap_to_value(hashmap: HashMap<String, Value>) -> Value {
        Value::Object(hashmap.into_iter().collect::<Map<String, Value>>())
    }
}
