pub mod constants;
pub mod directory;
pub mod error;
pub mod format;
pub mod logging;
pub mod markdown;
pub mod output;
pub mod progress;
pub mod registry;
pub mod table;
pub mod throttle;
pub mod ticker;
pub mod tree;
pub mod types;

use std::io::Write;

pub fn bell() {
    print!("\x07");
    std::io::stdout().flush().unwrap();
}

pub fn config_dir() -> std::path::PathBuf {
    let home = dirs::home_dir().expect("Failed to get home directory");
    let config_dir = home.join(".config").join("rwalk");
    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir).expect("Failed to create config directory");
    }
    config_dir
}
