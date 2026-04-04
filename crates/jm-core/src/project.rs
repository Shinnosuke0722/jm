use crate::error::Result;
use crate::java_version::VersionSpec;
use std::path::{Path, PathBuf};

/// Source of a detected Java version requirement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionSource {
    /// `JM_JAVA_VERSION` environment variable.
    EnvVar,
    /// `.java-version` file.
    JavaVersionFile(PathBuf),
    /// `.sdkmanrc` file.
    SdkmanRc(PathBuf),
    /// Global default from config.
    GlobalDefault,
}

/// Detected version requirement from project context.
#[derive(Debug, Clone)]
pub struct DetectedVersion {
    pub spec: VersionSpec,
    pub source: VersionSource,
}

/// Walk the directory tree from `start` upward, looking for version config files.
///
/// Returns the first match found, in priority order:
/// 1. `JM_JAVA_VERSION` environment variable
/// 2. `.java-version` file
/// 3. `.sdkmanrc` file
pub fn detect_version(start: &Path) -> Result<Option<DetectedVersion>> {
    // Check environment variable first
    if let Ok(val) = std::env::var("JM_JAVA_VERSION") {
        if !val.is_empty() {
            let spec = VersionSpec::parse(&val)?;
            return Ok(Some(DetectedVersion {
                spec,
                source: VersionSource::EnvVar,
            }));
        }
    }

    // Walk up the directory tree
    let mut dir = start.to_path_buf();
    loop {
        // Check .java-version
        let java_version_file = dir.join(".java-version");
        if java_version_file.exists() {
            let content = std::fs::read_to_string(&java_version_file)?;
            let content = content.trim();
            if !content.is_empty() {
                let spec = VersionSpec::parse(content)?;
                return Ok(Some(DetectedVersion {
                    spec,
                    source: VersionSource::JavaVersionFile(java_version_file),
                }));
            }
        }

        // Check .sdkmanrc
        let sdkmanrc = dir.join(".sdkmanrc");
        if sdkmanrc.exists() {
            if let Some(spec) = parse_sdkmanrc(&sdkmanrc)? {
                return Ok(Some(DetectedVersion {
                    spec,
                    source: VersionSource::SdkmanRc(sdkmanrc),
                }));
            }
        }

        // Move to parent directory
        if !dir.pop() {
            break;
        }
    }

    Ok(None)
}

/// Parse a `.sdkmanrc` file, looking for `java=<version>` line.
fn parse_sdkmanrc(path: &Path) -> Result<Option<VersionSpec>> {
    let content = std::fs::read_to_string(path)?;
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        if let Some(value) = line.strip_prefix("java=") {
            let value = value.trim();
            if !value.is_empty() {
                // SDKMAN uses suffixes like "21.0.2-tem", map them
                let spec = parse_sdkman_version(value)?;
                return Ok(Some(spec));
            }
        }
    }
    Ok(None)
}

/// Parse SDKMAN version format: `21.0.2-tem` -> distribution=temurin, version=21.0.2
fn parse_sdkman_version(s: &str) -> Result<VersionSpec> {
    if let Some(idx) = s.rfind('-') {
        let version_part = &s[..idx];
        let suffix = &s[idx + 1..];
        // Check if suffix looks like a distribution abbreviation (not a number)
        if !suffix.chars().next().is_none_or(|c| c.is_ascii_digit()) {
            let dist_str = map_sdkman_suffix(suffix);
            let combined = format!("{}-{}", dist_str, version_part);
            return VersionSpec::parse(&combined);
        }
    }
    VersionSpec::parse(s)
}

/// Map SDKMAN distribution suffixes to jm distribution names.
fn map_sdkman_suffix(suffix: &str) -> &str {
    match suffix {
        "tem" => "temurin",
        "amzn" => "corretto",
        "zulu" => "zulu",
        "oracle" => "oracle",
        "librca" => "liberica",
        "sapmchn" => "sapmachine",
        "sem" => "semeru",
        "graalce" => "graalvm-ce",
        "graal" => "graalvm",
        "ms" => "microsoft",
        "mandrel" => "mandrel",
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::distribution::Distribution;

    #[test]
    fn parse_sdkman_version_with_suffix() {
        let spec = parse_sdkman_version("21.0.2-tem").unwrap();
        assert_eq!(spec.distribution, Some(Distribution::Temurin));
        assert_eq!(spec.version.major, 21);
    }

    #[test]
    fn parse_sdkman_version_without_suffix() {
        let spec = parse_sdkman_version("21.0.2").unwrap();
        assert_eq!(spec.distribution, None);
        assert_eq!(spec.version.major, 21);
    }

    #[test]
    fn detect_from_java_version_file() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(".java-version"), "temurin-21\n").unwrap();

        let detected = detect_version(dir.path()).unwrap().unwrap();
        assert_eq!(detected.spec.distribution, Some(Distribution::Temurin));
        assert_eq!(detected.spec.version.major, 21);
        assert!(matches!(detected.source, VersionSource::JavaVersionFile(_)));
    }

    #[test]
    fn detect_from_sdkmanrc() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(".sdkmanrc"), "java=17.0.10-amzn\n").unwrap();

        let detected = detect_version(dir.path()).unwrap().unwrap();
        assert_eq!(detected.spec.distribution, Some(Distribution::Corretto));
        assert_eq!(detected.spec.version.major, 17);
    }
}
