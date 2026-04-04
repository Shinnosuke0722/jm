use crate::models::{DownloadInfo, JdkPackage};
use crate::provider::{JdkProvider, PackageQuery};
use jm_core::error::JmError;
use reqwest::Client;
use serde::Deserialize;

const ADOPTIUM_BASE_URL: &str = "https://api.adoptium.net/v3";

/// Client for the Adoptium API v3 (fallback, Temurin only).
pub struct AdoptiumClient {
    client: Client,
}

// --- Adoptium-specific response types ---

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct AvailableReleases {
    available_lts_releases: Vec<u32>,
    available_releases: Vec<u32>,
    most_recent_lts: u32,
    most_recent_feature_release: u32,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct AdoptiumAsset {
    binary: AdoptiumBinary,
    release_name: String,
    version: AdoptiumVersion,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct AdoptiumBinary {
    architecture: String,
    image_type: String,
    os: String,
    package: AdoptiumPackageInfo,
}

#[derive(Debug, Deserialize)]
struct AdoptiumPackageInfo {
    checksum: Option<String>,
    link: String,
    name: String,
    size: u64,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct AdoptiumVersion {
    major: u32,
    minor: u32,
    security: u32,
    #[serde(default)]
    build: u32,
    openjdk_version: String,
    semver: String,
}

impl AdoptiumClient {
    pub fn new() -> Result<Self, JmError> {
        Self::with_proxy(None)
    }

    pub fn with_proxy(proxy: Option<&str>) -> Result<Self, JmError> {
        let mut builder = Client::builder()
            .user_agent(format!("jm/{}", env!("CARGO_PKG_VERSION")))
            .timeout(std::time::Duration::from_secs(30));

        if let Some(proxy_url) = proxy {
            builder = builder
                .proxy(
                    reqwest::Proxy::all(proxy_url)
                        .map_err(|e| JmError::ApiError(format!("invalid proxy URL: {}", e)))?,
                );
        }

        let client = builder
            .build()
            .map_err(|e| JmError::ApiError(e.to_string()))?;
        Ok(Self { client })
    }

    /// Map Disco-style OS parameter to Adoptium parameter.
    fn map_os(os: &str) -> &str {
        match os {
            "macos" => "mac",
            other => other,
        }
    }

    /// Map Disco-style arch parameter to Adoptium parameter.
    fn map_arch(arch: &str) -> &str {
        match arch {
            "x64" => "x64",
            "aarch64" => "aarch64",
            other => other,
        }
    }

    async fn fetch_assets(
        &self,
        major_version: u32,
        os: &str,
        arch: &str,
    ) -> Result<Vec<AdoptiumAsset>, JmError> {
        let url = format!(
            "{}/assets/latest/{}/hotspot?os={}&architecture={}&image_type=jdk",
            ADOPTIUM_BASE_URL,
            major_version,
            Self::map_os(os),
            Self::map_arch(arch),
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| JmError::ApiError(format!("Adoptium request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(JmError::ApiError(format!(
                "Adoptium API returned status {}",
                response.status()
            )));
        }

        response
            .json()
            .await
            .map_err(|e| JmError::ApiError(format!("Adoptium parse error: {}", e)))
    }
}

#[async_trait::async_trait]
impl JdkProvider for AdoptiumClient {
    fn name(&self) -> &str {
        "Adoptium"
    }

    async fn query_packages(&self, query: &PackageQuery) -> Result<Vec<JdkPackage>, JmError> {
        let major = query
            .major_version
            .ok_or_else(|| JmError::ApiError("Adoptium requires a major version".into()))?;

        let os = query.operating_system.as_deref().unwrap_or("linux");
        let arch = query.architecture.as_deref().unwrap_or("x64");

        let assets = self.fetch_assets(major, os, arch).await?;

        Ok(assets
            .into_iter()
            .map(|a| {
                let is_lts = [7, 8, 11, 17, 21, 25].contains(&a.version.major);
                JdkPackage {
                    id: format!("adoptium-{}", a.version.semver),
                    distribution: "temurin".to_string(),
                    major_version: a.version.major,
                    java_version: a.version.openjdk_version,
                    operating_system: a.binary.os.clone(),
                    architecture: a.binary.architecture.clone(),
                    archive_type: if a.binary.package.name.ends_with(".zip") {
                        "zip".to_string()
                    } else {
                        "tar.gz".to_string()
                    },
                    filename: a.binary.package.name,
                    term_of_support: if is_lts {
                        "LTS".to_string()
                    } else {
                        "STS".to_string()
                    },
                    directly_downloadable: true,
                    size: a.binary.package.size,
                }
            })
            .collect())
    }

    async fn resolve_download(&self, package: &JdkPackage) -> Result<DownloadInfo, JmError> {
        // For Adoptium, the download info is already embedded in the asset query.
        // Re-fetch assets to get the link and checksum.
        let major = package.major_version;
        let os = &package.operating_system;
        let arch = &package.architecture;

        let assets = self.fetch_assets(major, os, arch).await?;
        let asset = assets
            .into_iter()
            .find(|a| a.binary.package.name == package.filename)
            .ok_or_else(|| {
                JmError::ApiError(format!("Adoptium asset not found for {}", package.filename))
            })?;

        Ok(DownloadInfo {
            url: asset.binary.package.link,
            checksum_sha256: asset.binary.package.checksum,
            filename: package.filename.clone(),
            size: asset.binary.package.size,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn os_mapping() {
        assert_eq!(AdoptiumClient::map_os("macos"), "mac");
        assert_eq!(AdoptiumClient::map_os("linux"), "linux");
        assert_eq!(AdoptiumClient::map_os("windows"), "windows");
    }

    #[test]
    fn arch_mapping() {
        assert_eq!(AdoptiumClient::map_arch("x64"), "x64");
        assert_eq!(AdoptiumClient::map_arch("aarch64"), "aarch64");
    }
}
