use std::{thread, time::Duration};

use rustic_config::{ConfigManagerBuilder, FilePath, ConfigSource, error::ConfigError};
fn main() -> Result<(), ConfigError> {
    let mut config_manager_builder = ConfigManagerBuilder::new();
    config_manager_builder
        .add_source(ConfigSource::File(FilePath::new("../test/test.json")));
    let config_manager = config_manager_builder
        .build()?;
    let (term_tx, term_rx) = oneshot::channel();
    let rx = config_manager.watch_file_changes(term_rx)?;

    thread::spawn(move || {
        for event in rx {
            println!("File Watcher Event: {:?}", event.unwrap())
        }
    });

    // Wait for 10 sec
    thread::sleep(Duration::from_secs(10));
    // When you want to stop watching the file
    term_tx.send(()).unwrap();

    Ok(())
}