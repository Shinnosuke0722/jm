use crate::distribution::Distribution;
use crate::error::{JmError, Result};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;

/// A parsed Java version number.
///
/// Handles both modern (JEP 223, Java 9+) and legacy (pre-9) formats:
/// - Modern: `21.0.2+13`, `17.0.10`, `25`
/// - Legacy: `1.8.0_362`, `1.7.0_80`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JavaVersion {
    pub major: u32,
    pub minor: Option<u32>,
    pub patch: Option<u32>,
    pub build: Option<String>,
}

impl JavaVersion {
    pub fn new(major: u32) -> Self {
        Self {
            major,
            minor: None,
            patch: None,
            build: None,
        }
    }

    /// Parse a Java version string.
    ///
    /// Supports:
    /// - `21`, `17`, `8`
    /// - `21.0.2`, `17.0.10`
    /// - `21.0.2+13`
    /// - `1.8.0_362` (legacy, normalized to major=8)
    pub fn parse(s: &str) -> Result<Self> {
        let s = s.trim();
        if s.is_empty() {
            return Err(JmError::InvalidVersion("empty version string".into()));
        }

        // Legacy format: 1.x.y_z
        if s.starts_with("1.") {
            return Self::parse_legacy(s);
        }

        // Modern format: major[.minor[.patch]][+build]
        let (version_part, build) = if let Some(idx) = s.find('+') {
            (&s[..idx], Some(s[idx + 1..].to_string()))
        } else {
            (s, None)
        };

        let parts: Vec<&str> = version_part.split('.').collect();
        let major = parts[0]
            .parse::<u32>()
            .map_err(|_| JmError::InvalidVersion(s.to_string()))?;
        let minor = parts
            .get(1)
            .map(|p| {
                p.parse::<u32>()
                    .map_err(|_| JmError::InvalidVersion(s.to_string()))
            })
            .transpose()?;
        let patch = parts
            .get(2)
            .map(|p| {
                p.parse::<u32>()
                    .map_err(|_| JmError::InvalidVersion(s.to_string()))
            })
            .transpose()?;

        if parts.len() > 3 {
            return Err(JmError::InvalidVersion(s.to_string()));
        }

        Ok(Self {
            major,
            minor,
            patch,
            build,
        })
    }

    fn parse_legacy(s: &str) -> Result<Self> {
        // 1.8.0_362 -> major=8, minor=0, patch=362
        let without_prefix = s.strip_prefix("1.").ok_or_else(|| {
            JmError::InvalidVersion(format!("expected 1.x prefix in legacy format: {}", s))
        })?;

        let parts: Vec<&str> = without_prefix.split(['.', '_']).collect();
        let major = parts[0]
            .parse::<u32>()
            .map_err(|_| JmError::InvalidVersion(s.to_string()))?;
        let minor = parts.get(1).and_then(|p| p.parse::<u32>().ok());
        let patch = parts.get(2).and_then(|p| p.parse::<u32>().ok());

        Ok(Self {
            major,
            minor,
            patch,
            build: None,
        })
    }

    /// Check if this version matches a version spec (which may be partial).
    ///
    /// A spec with only major version matches any version with the same major.
    /// More specific specs require matching on minor and patch as well.
    pub fn matches(&self, spec: &JavaVersion) -> bool {
        if self.major != spec.major {
            return false;
        }
        if let Some(minor) = spec.minor {
            if self.minor != Some(minor) {
                return false;
            }
        }
        if let Some(patch) = spec.patch {
            if self.patch != Some(patch) {
                return false;
            }
        }
        if let Some(ref build) = spec.build {
            if self.build.as_ref() != Some(build) {
                return false;
            }
        }
        true
    }
}

impl Ord for JavaVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        self.major
            .cmp(&other.major)
            .then_with(|| self.minor.unwrap_or(0).cmp(&other.minor.unwrap_or(0)))
            .then_with(|| self.patch.unwrap_or(0).cmp(&other.patch.unwrap_or(0)))
    }
}

impl PartialOrd for JavaVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for JavaVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.major)?;
        if let Some(minor) = self.minor {
            write!(f, ".{}", minor)?;
            if let Some(patch) = self.patch {
                write!(f, ".{}", patch)?;
            }
        }
        if let Some(ref build) = self.build {
            write!(f, "+{}", build)?;
        }
        Ok(())
    }
}

/// A user-specified version requirement, optionally qualified with a distribution.
///
/// Examples:
/// - `21` -> any distribution, major version 21
/// - `temurin-21` -> Temurin distribution, major version 21
/// - `temurin-21.0.2` -> Temurin, exact version
/// - `corretto-17.0.10.7.1` -> Corretto with distribution-specific version
#[derive(Debug, Clone)]
pub struct VersionSpec {
    pub distribution: Option<Distribution>,
    pub version: JavaVersion,
}

impl VersionSpec {
    /// Parse a user-supplied version spec string.
    ///
    /// Format: `[distribution-]version`
    pub fn parse(s: &str) -> Result<Self> {
        let s = s.trim();

        // Try to split on first `-` that precedes a digit sequence.
        // "temurin-21.0.2" -> distribution="temurin", version="21.0.2"
        // "21.0.2+13" -> no distribution
        // "graalvm-ce-22" -> distribution="graalvm-ce", version="22"
        if let Some((dist_part, ver_part)) = split_distribution_version(s) {
            let dist = Distribution::parse(dist_part);
            dist.validate()?;
            Ok(Self {
                distribution: Some(dist),
                version: JavaVersion::parse(ver_part)?,
            })
        } else {
            Ok(Self {
                distribution: None,
                version: JavaVersion::parse(s)?,
            })
        }
    }
}

impl fmt::Display for VersionSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref dist) = self.distribution {
            write!(f, "{}-{}", dist, self.version)
        } else {
            write!(f, "{}", self.version)
        }
    }
}

/// Split "temurin-21.0.2" into ("temurin", "21.0.2").
/// Returns None if the string is just a version (starts with a digit).
fn split_distribution_version(s: &str) -> Option<(&str, &str)> {
    // If it starts with a digit or "1." (legacy), it's a bare version.
    if s.starts_with(|c: char| c.is_ascii_digit()) {
        return None;
    }

    // Find the last `-` that is followed by a digit.
    for (i, _) in s.rmatch_indices('-') {
        let after = &s[i + 1..];
        if after.starts_with(|c: char| c.is_ascii_digit()) {
            return Some((&s[..i], after));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_modern_full() {
        let v = JavaVersion::parse("21.0.2+13").unwrap();
        assert_eq!(v.major, 21);
        assert_eq!(v.minor, Some(0));
        assert_eq!(v.patch, Some(2));
        assert_eq!(v.build.as_deref(), Some("13"));
    }

    #[test]
    fn parse_modern_major_only() {
        let v = JavaVersion::parse("21").unwrap();
        assert_eq!(v.major, 21);
        assert_eq!(v.minor, None);
        assert_eq!(v.patch, None);
    }

    #[test]
    fn parse_legacy() {
        let v = JavaVersion::parse("1.8.0_362").unwrap();
        assert_eq!(v.major, 8);
        assert_eq!(v.minor, Some(0));
        assert_eq!(v.patch, Some(362));
    }

    #[test]
    fn version_matching() {
        let full = JavaVersion::parse("21.0.2+13").unwrap();
        let spec_major = JavaVersion::parse("21").unwrap();
        let spec_minor = JavaVersion::parse("21.0").unwrap();
        let spec_exact = JavaVersion::parse("21.0.2").unwrap();
        let spec_wrong = JavaVersion::parse("17").unwrap();

        assert!(full.matches(&spec_major));
        assert!(full.matches(&spec_minor));
        assert!(full.matches(&spec_exact));
        assert!(!full.matches(&spec_wrong));
    }

    #[test]
    fn version_ordering() {
        let v17 = JavaVersion::parse("17.0.10").unwrap();
        let v21 = JavaVersion::parse("21.0.2").unwrap();
        let v21_old = JavaVersion::parse("21.0.1").unwrap();
        assert!(v17 < v21);
        assert!(v21_old < v21);
    }

    #[test]
    fn display_format() {
        assert_eq!(
            JavaVersion::parse("21.0.2+13").unwrap().to_string(),
            "21.0.2+13"
        );
        assert_eq!(JavaVersion::parse("21").unwrap().to_string(), "21");
        assert_eq!(
            JavaVersion::parse("17.0.10").unwrap().to_string(),
            "17.0.10"
        );
    }

    #[test]
    fn version_spec_with_distribution() {
        let spec = VersionSpec::parse("temurin-21.0.2").unwrap();
        assert_eq!(spec.distribution, Some(Distribution::Temurin));
        assert_eq!(spec.version.major, 21);
        assert_eq!(spec.version.minor, Some(0));
        assert_eq!(spec.version.patch, Some(2));
    }

    #[test]
    fn version_spec_bare_version() {
        let spec = VersionSpec::parse("21").unwrap();
        assert_eq!(spec.distribution, None);
        assert_eq!(spec.version.major, 21);
    }

    #[test]
    fn version_spec_compound_distribution() {
        let spec = VersionSpec::parse("graalvm-ce-22").unwrap();
        assert_eq!(spec.distribution, Some(Distribution::GraalVmCe));
        assert_eq!(spec.version.major, 22);
    }
}
