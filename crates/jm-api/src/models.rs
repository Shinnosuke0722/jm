use serde::{Deserialize, Serialize};

/// Wrapper for all Foojay Disco API responses.
#[derive(Debug, Clone, Deserialize)]
pub struct DiscoResponse<T> {
    pub result: Vec<T>,
    pub message: String,
}

/// A JDK distribution from the Disco `/distributions` endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct DiscoDistribution {
    pub name: String,
    pub api_parameter: String,
    pub maintained: bool,
    pub available: bool,
    pub build_of_openjdk: bool,
    pub build_of_graalvm: bool,
    pub official_uri: String,
    #[serde(default)]
    pub synonyms: Vec<String>,
    #[serde(default)]
    pub versions: Vec<String>,
}

/// A major Java version from the Disco `/major_versions` endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct DiscoMajorVersion {
    pub major_version: u32,
    pub term_of_support: String,
    pub maintained: bool,
    pub early_access_only: bool,
    pub release_status: String,
    #[serde(default)]
    pub versions: Vec<String>,
}

/// A JDK package from the Disco `/packages` endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct DiscoPackage {
    pub id: String,
    pub archive_type: String,
    pub distribution: String,
    pub major_version: u32,
    pub java_version: String,
    #[serde(default)]
    pub distribution_version: String,
    pub jdk_version: u32,
    #[serde(default)]
    pub latest_build_available: bool,
    pub release_status: String,
    pub term_of_support: String,
    pub operating_system: String,
    #[serde(default)]
    pub lib_c_type: String,
    pub architecture: String,
    pub package_type: String,
    #[serde(default)]
    pub javafx_bundled: bool,
    #[serde(default)]
    pub directly_downloadable: bool,
    pub filename: String,
    pub links: DiscoPackageLinks,
    #[serde(default)]
    pub free_use_in_production: bool,
    #[serde(default)]
    pub tck_tested: String,
    #[serde(default)]
    pub aqavit_certified: String,
    #[serde(default, deserialize_with = "deserialize_size")]
    pub size: u64,
}

fn deserialize_size<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v = i64::deserialize(deserializer).unwrap_or(0);
    Ok(if v > 0 { v as u64 } else { 0 })
}

#[derive(Debug, Clone, Deserialize)]
pub struct DiscoPackageLinks {
    pub pkg_info_uri: String,
    #[serde(default)]
    pub pkg_download_redirect: String,
}

/// Package detail from the Disco `/ids/{id}` endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct DiscoPackageInfo {
    pub filename: String,
    #[serde(default)]
    pub direct_download_uri: String,
    #[serde(default)]
    pub download_site_uri: String,
    #[serde(default)]
    pub signature_uri: String,
    #[serde(default)]
    pub checksum_uri: String,
    #[serde(default)]
    pub checksum: String,
    #[serde(default)]
    pub checksum_type: String,
}

/// Unified JDK package info (normalized from any API source).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JdkPackage {
    pub id: String,
    pub distribution: String,
    pub major_version: u32,
    pub java_version: String,
    pub operating_system: String,
    pub architecture: String,
    pub archive_type: String,
    pub filename: String,
    pub term_of_support: String,
    pub directly_downloadable: bool,
    pub size: u64,
}

/// Download info resolved for a specific package.
#[derive(Debug, Clone)]
pub struct DownloadInfo {
    pub url: String,
    pub checksum_sha256: Option<String>,
    pub filename: String,
    pub size: u64,
}
