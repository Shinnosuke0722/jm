use crate::models::{DownloadInfo, JdkPackage};
use jm_core::error::JmError;

/// Query parameters for searching JDK packages.
#[derive(Debug, Clone, Default)]
pub struct PackageQuery {
    pub major_version: Option<u32>,
    pub distribution: Option<String>,
    pub operating_system: Option<String>,
    pub architecture: Option<String>,
    pub archive_type: Option<String>,
    pub package_type: Option<String>,
    pub release_status: Option<String>,
    pub latest: Option<String>,
}

/// Trait for JDK package discovery and download resolution.
#[async_trait::async_trait]
pub trait JdkProvider: Send + Sync {
    /// Human-readable provider name.
    fn name(&self) -> &str;

    /// Search for available JDK packages matching the query.
    async fn query_packages(&self, query: &PackageQuery) -> Result<Vec<JdkPackage>, JmError>;

    /// Resolve the download URL and checksum for a specific package.
    async fn resolve_download(&self, package: &JdkPackage) -> Result<DownloadInfo, JmError>;
}
