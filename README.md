# Rustic Config

`rustic_config` is a Rust library designed for seamless and efficient configuration management in Rust applications. It elegantly handles configurations from various sources, focusing on the separation of operational and development settings.

## Key Features

- **Multiple Configuration Sources**: Supports JSON, YAML, environment variables, and command-line arguments.
- **Layered Configuration**: Implements a hierarchy with command-line arguments taking precedence over environment variables, which in turn override file configurations.
- **Separation of Concerns**: Distinct handling of operational (Ops) and development (Devs) configurations to cater to different needs and environments.
- **Flexible and Extensible**: Easily extendable for different sources or formats of configurations.
- **Simple API**: Intuitive functions for fetching and using configuration values.

## Installation

Add `rustic_config` to your `Cargo.toml`:

```toml
[dependencies]
rustic_config = "0.1.0"
```

## Usage

Basic usage example:

```rust
use rustic_config::{ConfigManagerBuilder, ConfigSource, FilePath, error::ConfigError};

fn main() -> Result<(), > {
    let mut builder = ConfigManagerBuilder::new();
    builder.add_source(ConfigSource::File(FilePath::new("config.yaml")));
    builder.add_source(ConfigSource::File(FilePath::new("config.yaml")));
    let config_manager = builder.build()?;

    let db_url: String = config_manager.get_string("database_url").unwrap();
    println!("Database URL: {}", db_url);

    Ok(())
}
```

## Design Principles
### Layered Configuration
The configuration management follows a specific hierarchy:

1. Command-Line Arguments (CLI): Highest precedence, ideal for developers to pass configuration during development or debugging.
2. Environment Variables (Env): For operational configurations, suited for changing settings in different deployment environments without altering the code.
3. Configuration Files (File): Base configurations, providing defaults or shared settings.

### Separation of Ops and Devs Configurations
rustic_config emphasizes the distinct needs of operations teams and developers:

* Ops Configurations: Focused on environment variables, allowing easy adaptation to different deployment environments without code changes.
* Devs Configurations: Leveraging CLI arguments for temporary or development-specific configurations without impacting the base or operational settings.

## Contributing
Contributions to rustic_config are welcome! Please read our contributing guidelines to get started.

## License
Licensed under MIT or Apache 2.0, at your option.