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
        let content =
            toml::to_string_pretty(self).map_err(|e| JmError::ConfigError(e.to_string()))?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, content)?;
        Ok(())
    }
}
