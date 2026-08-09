use crate::distribution::Distribution;
use crate::error::Result;
use crate::java_version::JavaVersion;
use crate::storage::StorageDirs;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Registry of all locally installed JDKs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registry {
    pub format_version: u32,
    pub installations: Vec<Installation>,
}

/// A single installed JDK entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Installation {
    /// Unique identifier: `<distribution>-<full_version>` (e.g., "temurin-21.0.2+13").
    pub id: String,
    /// The distribution name.
    pub distribution: Distribution,
    /// Parsed Java version.
    pub java_version: JavaVersion,
    /// Full version string as provided by the API.
    pub full_version: String,
    /// Major version number (for quick filtering).
    pub major_version: u32,
    /// Absolute path to the installed JDK directory.
    pub path: PathBuf,
    /// When this JDK was installed.
    pub installed_at: DateTime<Utc>,
    /// Whether this is an LTS release.
    pub is_lts: bool,
}

impl Registry {
    /// Load the registry from disk, or create an empty one if it doesn't exist.
    pub fn load(dirs: &StorageDirs) -> Result<Self> {
        let path = dirs.registry_path();
        Self::load_from(&path)
    }

    pub fn load_from(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::empty());
        }
        let content = std::fs::read_to_string(path)?;
        let registry: Self = serde_json::from_str(&content)?;
        Ok(registry)
    }

    /// Save the registry to disk.
    pub fn save(&self, dirs: &StorageDirs) -> Result<()> {
        let path = dirs.registry_path();
        self.save_to(&path)
    }

    pub fn save_to(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        // Atomic write: write to temp file, then rename
        let tmp_path = path.with_extension("json.tmp");
        std::fs::write(&tmp_path, &content)?;
        std::fs::rename(&tmp_path, path)?;
        Ok(())
    }

    /// Acquire an exclusive file lock, load the registry, apply the mutation,
    /// and save atomically. Prevents concurrent processes from corrupting data.
    pub fn locked_update<F, T>(dirs: &StorageDirs, f: F) -> Result<T>
    where
        F: FnOnce(&mut Self) -> Result<T>,
    {
        let path = dirs.registry_path();
        let lock_path = path.with_extension("lock");
        if let Some(parent) = lock_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let lock_file = std::fs::File::create(&lock_path)?;
        lock_file.lock()?;

        let mut registry = Self::load_from(&path)?;
        let result = f(&mut registry)?;
        registry.save_to(&path)?;

        // Lock released on drop
        Ok(result)
    }

    pub fn empty() -> Self {
        Self {
            format_version: 1,
            installations: Vec::new(),
        }
    }

    /// Add a new installation entry.
    pub fn add(&mut self, installation: Installation) {
        // Remove existing entry with the same id if present (reinstall)
        self.installations.retain(|i| i.id != installation.id);
        self.installations.push(installation);
    }

    /// Remove an installation by id.
    pub fn remove(&mut self, id: &str) -> Option<Installation> {
        if let Some(pos) = self.installations.iter().position(|i| i.id == id) {
            Some(self.installations.remove(pos))
        } else {
            None
        }
    }

    /// Find an installation by exact id.
    pub fn find_by_id(&self, id: &str) -> Option<&Installation> {
        self.installations.iter().find(|i| i.id == id)
    }

    /// Find installations matching a version spec (partial match).
    /// For example, "21" matches "temurin-21.0.2+13" and "corretto-21.0.3".
    pub fn find_matching(
        &self,
        distribution: Option<&Distribution>,
        version: &JavaVersion,
    ) -> Vec<&Installation> {
        self.installations
            .iter()
            .filter(|inst| {
                if let Some(dist) = distribution
                    && &inst.distribution != dist
                {
                    return false;
                }
                inst.java_version.matches(version)
            })
            .collect()
    }

    /// Get all installations sorted by version (newest first).
    pub fn sorted(&self) -> Vec<&Installation> {
        let mut sorted: Vec<_> = self.installations.iter().collect();
        sorted.sort_by(|a, b| b.java_version.cmp(&a.java_version));
        sorted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_installation(id: &str, major: u32) -> Installation {
        let dist_name = id.split('-').next().unwrap_or("temurin");
        Installation {
            id: id.to_string(),
            distribution: Distribution::parse(dist_name),
            java_version: JavaVersion {
                major,
                minor: Some(0),
                patch: Some(2),
                build: None,
            },
            full_version: format!("{major}.0.2"),
            major_version: major,
            path: PathBuf::from(format!("/tmp/jdks/{id}")),
            installed_at: Utc::now(),
            is_lts: major == 21 || major == 17,
        }
    }

    #[test]
    fn add_and_find() {
        let mut reg = Registry::empty();
        reg.add(make_installation("temurin-21.0.2", 21));
        reg.add(make_installation("temurin-17.0.10", 17));

        assert_eq!(reg.installations.len(), 2);
        assert!(reg.find_by_id("temurin-21.0.2").is_some());
        assert!(reg.find_by_id("nonexistent").is_none());
    }

    #[test]
    fn remove() {
        let mut reg = Registry::empty();
        reg.add(make_installation("temurin-21.0.2", 21));
        assert_eq!(reg.installations.len(), 1);
        reg.remove("temurin-21.0.2");
        assert_eq!(reg.installations.len(), 0);
    }

    #[test]
    fn find_matching_by_major() {
        let mut reg = Registry::empty();
        reg.add(make_installation("temurin-21.0.2", 21));
        reg.add(make_installation("corretto-21.0.3", 21));
        reg.add(make_installation("temurin-17.0.10", 17));

        let spec = JavaVersion::new(21);
        let matches = reg.find_matching(None, &spec);
        assert_eq!(matches.len(), 2);

        let matches = reg.find_matching(Some(&Distribution::Temurin), &spec);
        assert_eq!(matches.len(), 1);
    }
}
