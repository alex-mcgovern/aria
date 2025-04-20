mod error;
mod models;

pub use error::ConfigError;
pub use models::{Config, ProviderType};

use std::env;
use std::fs;
use std::path::Path;

/// Attempts to load the configuration from a file.
/// First checks the current working directory, then falls back to ~/.config/aria/aria.yml
pub fn load_config_file() -> Result<Config, ConfigError> {
    // Try current working directory first
    let cwd_config = env::current_dir()?.join("aria.yml");

    // Then try the ~/.config/aria/aria.yml path
    let home_dir = dirs::home_dir().ok_or(ConfigError::NotFound)?;
    let home_config = home_dir.join(".config").join("aria").join("aria.yml");

    // Try loading from the CWD config first, then fall back to home config
    let config_path = if cwd_config.exists() {
        cwd_config
    } else if home_config.exists() {
        home_config
    } else {
        return Err(ConfigError::NotFound);
    };

    let path: &Path = &config_path;
    let contents = fs::read_to_string(path)?;
    let config: Config = serde_yaml::from_str(&contents)?;
    Ok(config)
}
