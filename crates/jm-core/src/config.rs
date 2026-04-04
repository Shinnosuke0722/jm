use crate::error::{JmError, Result};
use crate::storage::StorageDirs;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Global jm configuration (stored in config.toml).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
#[derive(Default)]
pub struct Config {
    pub global: GlobalConfig,
    pub install: InstallConfig,
    pub api: ApiConfig,
    pub ui: UiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GlobalConfig {
    /// Default JDK version when no project-level override exists.
    pub java_version: Option<String>,
    /// Preferred distribution when version is specified without one.
    pub preferred_distribution: String,
    /// Auto-install missing versions when auto-switching.
    pub auto_install: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct InstallConfig {
    /// Verify checksums on download.
    pub verify_checksum: bool,
    /// Keep downloaded archives in cache.
    pub keep_archives: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ApiConfig {
    /// Primary API endpoint.
    pub disco_api_url: String,
    /// Fallback to Adoptium API if Disco is unavailable.
    pub fallback_enabled: bool,
    /// HTTP timeout in seconds.
    pub timeout: u64,
    /// Cache API responses for this many hours.
    pub cache_ttl_hours: u64,
    /// Optional HTTP proxy.
    pub proxy: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UiConfig {
    /// Enable colored output: "auto", "always", "never".
    pub color: String,
    /// Show progress bars during downloads.
    pub progress: bool,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            java_version: None,
            preferred_distribution: "temurin".to_string(),
            auto_install: false,
        }
    }
}

impl Default for InstallConfig {
    fn default() -> Self {
        Self {
            verify_checksum: true,
            keep_archives: false,
        }
    }
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            disco_api_url: "https://api.foojay.io/disco/v3.0".to_string(),
            fallback_enabled: true,
            timeout: 30,
            cache_ttl_hours: 24,
            proxy: None,
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            color: "auto".to_string(),
            progress: true,
        }
    }
}

impl Config {
    /// Load configuration from file, or return defaults if the file doesn't exist.
    pub fn load(dirs: &StorageDirs) -> Result<Self> {
        let path = dirs.config_path();
        Self::load_from(&path)
    }

    pub fn load_from(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(path)?;
        toml::from_str(&content).map_err(|e| JmError::ConfigError(e.to_string()))
    }

    /// Save configuration to file.
    pub fn save(&self, dirs: &StorageDirs) -> Result<()> {
        let path = dirs.config_path();
        self.save_to(&path)
    }

    pub fn save_to(&self, path: &std::path::Path) -> Result<()> {
        let content =
            toml::to_string_pretty(self).map_err(|e| JmError::ConfigError(e.to_string()))?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let config = Config::default();
        assert_eq!(config.global.preferred_distribution, "temurin");
        assert!(!config.global.auto_install);
        assert!(config.global.java_version.is_none());
        assert!(config.install.verify_checksum);
        assert!(!config.install.keep_archives);
        assert!(config.api.fallback_enabled);
        assert_eq!(config.api.timeout, 30);
        assert_eq!(config.api.cache_ttl_hours, 24);
        assert!(config.api.proxy.is_none());
        assert_eq!(config.ui.color, "auto");
        assert!(config.ui.progress);
    }

    #[test]
    fn load_missing_file_returns_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nonexistent.toml");
        let config = Config::load_from(&path).unwrap();
        assert_eq!(config.global.preferred_distribution, "temurin");
    }

    #[test]
    fn save_and_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");

        let mut config = Config::default();
        config.global.preferred_distribution = "corretto".to_string();
        config.global.auto_install = true;
        config.api.proxy = Some("http://proxy:8080".to_string());
        config.api.timeout = 60;
        config.install.keep_archives = true;

        config.save_to(&path).unwrap();
        let loaded = Config::load_from(&path).unwrap();

        assert_eq!(loaded.global.preferred_distribution, "corretto");
        assert!(loaded.global.auto_install);
        assert_eq!(loaded.api.proxy, Some("http://proxy:8080".to_string()));
        assert_eq!(loaded.api.timeout, 60);
        assert!(loaded.install.keep_archives);
    }

    #[test]
    fn load_partial_toml_fills_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        // Write only one section
        std::fs::write(&path, "[global]\npreferred_distribution = \"zulu\"\n").unwrap();

        let config = Config::load_from(&path).unwrap();
        assert_eq!(config.global.preferred_distribution, "zulu");
        // Unset fields should be defaults
        assert!(config.api.fallback_enabled);
        assert_eq!(config.api.timeout, 30);
        assert!(config.ui.progress);
    }

    #[test]
    fn load_invalid_toml_returns_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "not valid toml {{{{").unwrap();

        let result = Config::load_from(&path);
        assert!(result.is_err());
    }

    #[test]
    fn save_creates_parent_dirs() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested").join("dir").join("config.toml");
        let config = Config::default();
        config.save_to(&path).unwrap();
        assert!(path.exists());
    }
}
