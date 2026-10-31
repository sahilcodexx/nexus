use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Configuration error types
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML deserialize error: {0}")]
    TomlDe(#[from] toml::de::Error),
    #[error("TOML error: {0}")]
    Toml(#[from] toml::ser::Error),
}

/// Application configuration loaded from TOML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Whether private mode is enabled
    pub private_mode: bool,
    /// Maximum number of clipboard entries to keep
    pub max_history: usize,
    /// Retention period in days (None for indefinite)
    pub retention_days: Option<i64>,
    /// Whether to automatically exclude sensitive applications
    pub auto_exclude_sensitive: bool,
    /// Excluded applications list
    pub excluded_apps: Vec<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            private_mode: false,
            max_history: 100,
            retention_days: None,
            auto_exclude_sensitive: false,
            excluded_apps: vec![
                "1password".to_string(),
                "bitwarden".to_string(),
                "keepassxc".to_string(),
            ],
        }
    }
}

impl AppConfig {
    /// Get the default config directory
    pub fn config_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("nexus")
    }

    /// Get the config file path
    pub fn config_file() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    /// Load configuration from default location
    pub fn load_default() -> Result<Self, ConfigError> {
        Self::load(&Self::config_file())
    }

    /// Load configuration from a file path
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        if !path.exists() {
            // Create default config
            let config = AppConfig::default();
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            config.save(path)?;
            return Ok(config);
        }

        let config_str = std::fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&config_str)?;
        Ok(config)
    }

    /// Save configuration to a file path
    pub fn save(&self, path: &Path) -> Result<(), ConfigError> {
        let toml = toml::to_string(self)?;
        std::fs::write(path, toml)?;
        Ok(())
    }

    /// Save to default location
    pub fn save_default(&self) -> Result<(), ConfigError> {
        std::fs::create_dir_all(Self::config_dir())?;
        self.save(&Self::config_file())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert!(!config.private_mode);
        assert_eq!(config.max_history, 100);
        assert!(config.retention_days.is_none());
        assert!(!config.excluded_apps.is_empty());
    }

    #[test]
    fn test_config_load_save() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        let config = AppConfig::default();
        config.save(&config_path).unwrap();

        let loaded = AppConfig::load(&config_path).unwrap();
        assert_eq!(loaded.private_mode, false);
        assert_eq!(loaded.max_history, 100);
    }
}