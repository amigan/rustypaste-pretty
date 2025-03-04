use crate::mime::MimeMatcher;
use crate::random::RandomURLConfig;
use byte_unit::Byte;
use config::{self, ConfigError};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Configuration values.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Config {
    /// Configuration settings.
    #[serde(rename = "config")]
    pub settings: Option<Settings>,
    /// Server configuration.
    pub server: ServerConfig,
    /// Paste configuration.
    pub paste: PasteConfig,
}

/// General settings for configuration.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Settings {
    /// Refresh rate of the configuration file.
    #[serde(with = "humantime_serde")]
    pub refresh_rate: Duration,
}

/// Server configuration.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ServerConfig {
    /// The socket address to bind.
    pub address: String,
    /// URL that can be used to access the server externally.
    pub url: Option<String>,
    /// Number of workers to start.
    pub workers: Option<usize>,
    /// Maximum content length.
    pub max_content_length: Byte,
    /// Storage path.
    pub upload_path: PathBuf,
    /// Request timeout.
    #[serde(default, with = "humantime_serde")]
    pub timeout: Option<Duration>,
    /// Authentication token.
    pub auth_token: Option<String>,
    /// Landing page text.
    pub landing_page: Option<String>,
    /// Expose version.
    pub expose_version: Option<bool>,
    /// Default to pretty when Accept: allows it.
    pub pretty_default: Option<bool>,
    /// Highlight.js style
    pub style: Option<String>,
}

/// Paste configuration.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct PasteConfig {
    /// Random URL configuration.
    pub random_url: RandomURLConfig,
    /// Default file extension.
    pub default_extension: String,
    /// Media type override options.
    #[serde(default)]
    pub mime_override: Vec<MimeMatcher>,
    /// Media type blacklist.
    #[serde(default)]
    pub mime_blacklist: Vec<String>,
    /// Allow duplicate uploads.
    pub duplicate_files: Option<bool>,
    /// Default expiry time.
    #[serde(default, with = "humantime_serde")]
    pub default_expiry: Option<Duration>,
    /// Delete expired files.
    pub delete_expired_files: Option<CleanupConfig>,
    /// Highlight override.
    #[serde(default)]
    pub highlight_override: Option<HashMap<String, String>>,
}

/// Cleanup configuration.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct CleanupConfig {
    /// Enable cleaning up.
    pub enabled: bool,
    /// Interval between clean-ups.
    #[serde(default, with = "humantime_serde")]
    pub interval: Duration,
}

impl Config {
    /// Parses the config file and returns the values.
    pub fn parse(path: &Path) -> Result<Config, ConfigError> {
        config::Config::builder()
            .set_default("style", "default").unwrap()
            .add_source(config::File::from(path))
            .add_source(config::Environment::default().separator("__"))
            .build()?
            .try_deserialize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_parse_config() -> Result<(), ConfigError> {
        let config_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config.toml");
        env::set_var("SERVER__ADDRESS", "0.0.1.1");
        let config = Config::parse(&config_path)?;
        assert_eq!("0.0.1.1", config.server.address);
        Ok(())
    }
}
