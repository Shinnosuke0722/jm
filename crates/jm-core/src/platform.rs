use crate::error::{JmError, Result};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Operating system identifier matching the Foojay Disco API parameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Os {
    Linux,
    #[serde(rename = "macos")]
    MacOs,
    Windows,
}

impl Os {
    /// Detect the current operating system.
    pub fn current() -> Result<Self> {
        match std::env::consts::OS {
            "linux" => Ok(Self::Linux),
            "macos" => Ok(Self::MacOs),
            "windows" => Ok(Self::Windows),
            other => Err(JmError::UnsupportedPlatform {
                os: other.to_string(),
                arch: std::env::consts::ARCH.to_string(),
            }),
        }
    }

    /// The Foojay Disco API parameter value.
    pub fn api_parameter(&self) -> &str {
        match self {
            Self::Linux => "linux",
            Self::MacOs => "macos",
            Self::Windows => "windows",
        }
    }

    /// Default archive type for this platform.
    pub fn default_archive_type(&self) -> &str {
        match self {
            Self::Linux | Self::MacOs => "tar.gz",
            Self::Windows => "zip",
        }
    }

    /// Name of the java executable.
    pub fn java_binary_name(&self) -> &str {
        match self {
            Self::Windows => "java.exe",
            _ => "java",
        }
    }
}

impl fmt::Display for Os {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.api_parameter())
    }
}

/// CPU architecture identifier matching the Foojay Disco API parameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Arch {
    X64,
    Aarch64,
}

impl Arch {
    /// Detect the current CPU architecture.
    pub fn current() -> Result<Self> {
        match std::env::consts::ARCH {
            "x86_64" => Ok(Self::X64),
            "aarch64" => Ok(Self::Aarch64),
            other => Err(JmError::UnsupportedPlatform {
                os: std::env::consts::OS.to_string(),
                arch: other.to_string(),
            }),
        }
    }

    /// The Foojay Disco API parameter value.
    pub fn api_parameter(&self) -> &str {
        match self {
            Self::X64 => "x64",
            Self::Aarch64 => "aarch64",
        }
    }
}

impl fmt::Display for Arch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.api_parameter())
    }
}

/// Represents the current platform (OS + architecture).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Platform {
    pub os: Os,
    pub arch: Arch,
}

impl Platform {
    /// Detect the current platform.
    pub fn current() -> Result<Self> {
        Ok(Self {
            os: Os::current()?,
            arch: Arch::current()?,
        })
    }
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", self.os, self.arch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_current_platform() {
        let platform = Platform::current().unwrap();
        // We should at least get a valid result on the test machine
        assert!(!platform.to_string().is_empty());
    }

    #[test]
    fn archive_types() {
        assert_eq!(Os::Linux.default_archive_type(), "tar.gz");
        assert_eq!(Os::MacOs.default_archive_type(), "tar.gz");
        assert_eq!(Os::Windows.default_archive_type(), "zip");
    }
}
