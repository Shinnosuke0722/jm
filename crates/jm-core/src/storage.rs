use crate::error::{JmError, Result};
use std::path::PathBuf;

/// Manages the directory layout for jm data, config, and cache.
///
/// If `JM_HOME` is set, all directories collapse under that single root.
/// Otherwise, follows platform conventions (XDG on Linux, Library on macOS,
/// AppData on Windows).
#[derive(Debug, Clone)]
pub struct StorageDirs {
    pub data_dir: PathBuf,
    pub config_dir: PathBuf,
    pub cache_dir: PathBuf,
}

impl StorageDirs {
    /// Resolve storage directories, respecting `JM_HOME` override.
    pub fn resolve() -> Result<Self> {
        let home = std::env::var("JM_HOME").ok().map(PathBuf::from);
        Self::resolve_with_home(home)
    }

    fn resolve_with_home(home: Option<PathBuf>) -> Result<Self> {
        if let Some(root) = home {
            return Ok(Self {
                data_dir: root.clone(),
                config_dir: root.clone(),
                cache_dir: root.join("cache"),
            });
        }

        let data_dir = dirs::data_dir()
            .ok_or_else(|| JmError::ConfigError("cannot determine data directory".into()))?
            .join("jm");

        let config_dir = dirs::config_dir()
            .ok_or_else(|| JmError::ConfigError("cannot determine config directory".into()))?
            .join("jm");

        let cache_dir = dirs::cache_dir()
            .ok_or_else(|| JmError::ConfigError("cannot determine cache directory".into()))?
            .join("jm");

        Ok(Self {
            data_dir,
            config_dir,
            cache_dir,
        })
    }

    /// Directory containing all installed JDKs.
    pub fn jdks_dir(&self) -> PathBuf {
        self.data_dir.join("jdks")
    }

    /// Symlink (or junction on Windows) pointing to the current global default JDK.
    pub fn current_link(&self) -> PathBuf {
        self.data_dir.join("current")
    }

    /// Path to the installed-JDK registry file.
    pub fn registry_path(&self) -> PathBuf {
        self.data_dir.join("registry.json")
    }

    /// Path to the global configuration file.
    pub fn config_path(&self) -> PathBuf {
        self.config_dir.join("config.toml")
    }

    /// Directory for cached API responses.
    pub fn api_cache_dir(&self) -> PathBuf {
        self.cache_dir.join("api")
    }

    /// Directory for downloaded archives (before extraction).
    pub fn downloads_dir(&self) -> PathBuf {
        self.cache_dir.join("downloads")
    }

    /// Ensure all required directories exist.
    pub fn ensure_dirs(&self) -> Result<()> {
        let dirs = [
            &self.data_dir,
            &self.config_dir,
            &self.cache_dir,
            &self.jdks_dir(),
            &self.api_cache_dir(),
            &self.downloads_dir(),
        ];
        for dir in dirs {
            std::fs::create_dir_all(dir)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jm_home_override() {
        let dir = tempfile::tempdir().unwrap();
        let dirs = StorageDirs::resolve_with_home(Some(dir.path().to_path_buf())).unwrap();
        assert_eq!(dirs.data_dir, dir.path());
        assert_eq!(dirs.config_dir, dir.path());
        assert_eq!(dirs.cache_dir, dir.path().join("cache"));
    }
}
