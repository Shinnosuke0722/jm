use thiserror::Error;

#[derive(Debug, Error)]
pub enum JmError {
    #[error("JDK version not found: {0}")]
    VersionNotFound(String),

    #[error("JDK not installed: {0}")]
    NotInstalled(String),

    #[error("JDK already installed: {0}")]
    AlreadyInstalled(String),

    #[error("Unknown distribution: {0}")]
    UnknownDistribution(String),

    #[error("Invalid version format: {0}")]
    InvalidVersion(String),

    #[error("Unsupported platform: {os}/{arch}")]
    UnsupportedPlatform { os: String, arch: String },

    #[error("Checksum mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch { expected: String, actual: String },

    #[error("Download failed: {0}")]
    DownloadFailed(String),

    #[error("Extraction failed: {0}")]
    ExtractionFailed(String),

    #[error("Could not find JDK home in archive (no bin/java found)")]
    JdkHomeNotFound,

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, JmError>;
