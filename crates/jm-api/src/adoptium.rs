use crate::models::{DownloadInfo, JdkPackage};
use crate::provider::{JdkProvider, PackageQuery};
use jm_core::error::JmError;
use jm_core::java_version::JavaVersion;
use reqwest::Client;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use url::Url;

const ADOPTIUM_BASE_URL: &str = "https://api.adoptium.net/v3";
const ADOPTIUM_PAGE_SIZE: usize = 20;

/// Client for the Adoptium API v3 (fallback, Temurin only).
pub struct AdoptiumClient {
    client: Client,
    base_url: String,
}

// --- Adoptium-specific response types ---

#[derive(Debug, Deserialize)]
struct AdoptiumLatestAsset {
    binary: AdoptiumBinary,
    release_name: String,
    version: AdoptiumVersion,
}

#[derive(Debug, Deserialize)]
struct AdoptiumRelease {
    binaries: Vec<AdoptiumBinary>,
    release_name: String,
    version_data: AdoptiumVersion,
}

#[derive(Debug)]
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

#[derive(Clone, Debug, Deserialize)]
struct AdoptiumVersion {
    major: u32,
    minor: u32,
    security: u32,
    #[serde(default)]
    build: u32,
}

impl AdoptiumClient {
    pub fn new() -> Result<Self, JmError> {
        Self::with_proxy(None)
    }

    pub fn with_proxy(proxy: Option<&str>) -> Result<Self, JmError> {
        Self::with_base_url(ADOPTIUM_BASE_URL.to_string(), proxy)
    }

    fn with_base_url(base_url: String, proxy: Option<&str>) -> Result<Self, JmError> {
        let mut builder = Client::builder()
            .tls_backend_rustls()
            .user_agent(format!("jm/{}", env!("CARGO_PKG_VERSION")))
            .timeout(std::time::Duration::from_secs(30));

        if let Some(proxy_url) = proxy {
            builder = builder.proxy(
                reqwest::Proxy::all(proxy_url)
                    .map_err(|e| JmError::ApiError(format!("invalid proxy URL: {}", e)))?,
            );
        }

        let client = builder
            .build()
            .map_err(|e| JmError::ApiError(e.to_string()))?;
        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
        })
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

    fn normalized_version(version: &AdoptiumVersion) -> String {
        format!(
            "{}.{}.{}+{}",
            version.major, version.minor, version.security, version.build
        )
    }

    fn version_range(requirement: &str) -> Result<String, JmError> {
        let version = JavaVersion::parse(requirement)
            .map_err(|error| JmError::ApiError(format!("invalid Adoptium version: {error}")))?;

        let (lower, upper) = if let Some(patch) = version.patch {
            let upper_patch = patch.checked_add(1).ok_or_else(|| {
                JmError::ApiError(format!(
                    "Adoptium version patch is too large: {requirement}"
                ))
            })?;
            let minor = version.minor.unwrap_or(0);
            (
                format!("{}.{}.{}", version.major, minor, patch),
                format!("{}.{}.{}", version.major, minor, upper_patch),
            )
        } else if let Some(minor) = version.minor {
            let upper_minor = minor.checked_add(1).ok_or_else(|| {
                JmError::ApiError(format!(
                    "Adoptium version minor is too large: {requirement}"
                ))
            })?;
            (
                format!("{}.{}", version.major, minor),
                format!("{}.{}", version.major, upper_minor),
            )
        } else {
            let upper_major = version.major.checked_add(1).ok_or_else(|| {
                JmError::ApiError(format!(
                    "Adoptium version major is too large: {requirement}"
                ))
            })?;
            (version.major.to_string(), upper_major.to_string())
        };

        Ok(format!("[{lower},{upper})"))
    }

    fn latest_url(&self, major_version: u32, os: &str, arch: &str) -> Result<Url, JmError> {
        let mut url = Url::parse(&format!(
            "{}/assets/latest/{}/hotspot",
            self.base_url, major_version
        ))
        .map_err(|error| JmError::ApiError(format!("invalid Adoptium URL: {error}")))?;
        url.query_pairs_mut()
            .append_pair("os", Self::map_os(os))
            .append_pair("architecture", Self::map_arch(arch))
            .append_pair("image_type", "jdk")
            .append_pair("jvm_impl", "hotspot")
            .append_pair("project", "jdk")
            .append_pair("release_type", "ga")
            .append_pair("vendor", "eclipse");
        Ok(url)
    }

    fn version_url(&self, range: &str, os: &str, arch: &str, page: usize) -> Result<Url, JmError> {
        let encoded_range: String =
            url::form_urlencoded::byte_serialize(range.as_bytes()).collect();
        let mut url = Url::parse(&format!(
            "{}/assets/version/{}",
            self.base_url, encoded_range
        ))
        .map_err(|error| JmError::ApiError(format!("invalid Adoptium URL: {error}")))?;
        url.query_pairs_mut()
            .append_pair("os", Self::map_os(os))
            .append_pair("architecture", Self::map_arch(arch))
            .append_pair("image_type", "jdk")
            .append_pair("jvm_impl", "hotspot")
            .append_pair("project", "jdk")
            .append_pair("release_type", "ga")
            .append_pair("vendor", "eclipse")
            .append_pair("page", &page.to_string())
            .append_pair("page_size", &ADOPTIUM_PAGE_SIZE.to_string())
            .append_pair("sort_method", "DEFAULT")
            .append_pair("sort_order", "DESC");
        Ok(url)
    }

    async fn get_json<T: DeserializeOwned + Default>(&self, url: Url) -> Result<T, JmError> {
        let response = self
            .client
            .get(url.clone())
            .send()
            .await
            .map_err(|e| JmError::ApiError(format!("Adoptium request failed: {e}")))?;

        // Adoptium represents a valid query with no matching releases as 404.
        // Providers expose an empty package set for that condition.
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(T::default());
        }
        if !response.status().is_success() {
            return Err(JmError::ApiError(format!(
                "Adoptium API returned status {} for {}",
                response.status(),
                url
            )));
        }

        response
            .json()
            .await
            .map_err(|e| JmError::ApiError(format!("Adoptium parse error: {e}")))
    }

    async fn fetch_latest_assets(
        &self,
        major_version: u32,
        os: &str,
        arch: &str,
    ) -> Result<Vec<AdoptiumAsset>, JmError> {
        let latest: Vec<AdoptiumLatestAsset> = self
            .get_json(self.latest_url(major_version, os, arch)?)
            .await?;
        Ok(latest
            .into_iter()
            .map(|asset| AdoptiumAsset {
                binary: asset.binary,
                release_name: asset.release_name,
                version: asset.version,
            })
            .collect())
    }

    async fn fetch_version_assets(
        &self,
        requirement: &str,
        os: &str,
        arch: &str,
    ) -> Result<Vec<AdoptiumAsset>, JmError> {
        let requested = JavaVersion::parse(requirement)
            .map_err(|error| JmError::ApiError(format!("invalid Adoptium version: {error}")))?;
        let range = Self::version_range(requirement)?;
        let mut page = 0;

        loop {
            let releases: Vec<AdoptiumRelease> = self
                .get_json(self.version_url(&range, os, arch, page)?)
                .await?;
            let release_count = releases.len();
            let mut matches = Vec::new();

            for release in releases {
                let normalized = Self::normalized_version(&release.version_data);
                let version = JavaVersion::parse(&normalized).map_err(|error| {
                    JmError::ApiError(format!("invalid version returned by Adoptium: {error}"))
                })?;
                if !version.matches(&requested) {
                    continue;
                }

                for binary in release.binaries {
                    matches.push(AdoptiumAsset {
                        binary,
                        release_name: release.release_name.clone(),
                        version: release.version_data.clone(),
                    });
                }
            }

            if !matches.is_empty() || release_count < ADOPTIUM_PAGE_SIZE {
                return Ok(matches);
            }
            page += 1;
        }
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

        let assets = match query.version_requirement.as_deref() {
            Some(requirement) => self.fetch_version_assets(requirement, os, arch).await?,
            None => self.fetch_latest_assets(major, os, arch).await?,
        };

        Ok(assets
            .into_iter()
            .map(|a| {
                let is_lts = [7, 8, 11, 17, 21, 25].contains(&a.version.major);
                JdkPackage {
                    id: format!("adoptium-{}", a.release_name),
                    distribution: "temurin".to_string(),
                    major_version: a.version.major,
                    java_version: Self::normalized_version(&a.version),
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
        // Re-fetch the selected version rather than `latest`: a new release may
        // appear between package selection and download, and historical
        // selections must remain resolvable.
        let os = &package.operating_system;
        let arch = &package.architecture;

        let assets = self
            .fetch_version_assets(&package.java_version, os, arch)
            .await?;
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
    use serde_json::{Value, json};
    use wiremock::matchers::{method, path, path_regex, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn version_json(major: u32, minor: u32, security: u32, build: u32) -> Value {
        json!({
            "major": major,
            "minor": minor,
            "security": security,
            "build": build,
            "openjdk_version": format!("{major}.{minor}.{security}+{build}-LTS"),
            "semver": format!("{major}.{minor}.{security}+{build}.0.LTS")
        })
    }

    fn binary_json(filename: &str, url: &str) -> Value {
        json!({
            "architecture": "x64",
            "image_type": "jdk",
            "os": "windows",
            "package": {
                "checksum": "abcdef1234567890",
                "link": url,
                "name": filename,
                "size": 123456
            }
        })
    }

    fn release_json(major: u32, minor: u32, security: u32, build: u32) -> Value {
        let filename = format!(
            "OpenJDK{major}U-jdk_x64_windows_hotspot_{major}.{minor}.{security}_{build}.zip"
        );
        json!({
            "binaries": [binary_json(&filename, &format!("https://example.com/{filename}"))],
            "release_name": format!("jdk-{major}.{minor}.{security}+{build}"),
            "version_data": version_json(major, minor, security, build)
        })
    }

    fn query(requirement: Option<&str>) -> PackageQuery {
        PackageQuery {
            major_version: Some(21),
            version_requirement: requirement.map(str::to_string),
            operating_system: Some("windows".to_string()),
            architecture: Some("x64".to_string()),
            ..Default::default()
        }
    }

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

    #[test]
    fn normalizes_structured_versions_and_builds_ranges() {
        let java_21 = AdoptiumVersion {
            major: 21,
            minor: 0,
            security: 2,
            build: 13,
        };
        let java_8: AdoptiumVersion = serde_json::from_value(json!({
            "major": 8,
            "minor": 0,
            "security": 502,
            "build": 7,
            "openjdk_version": "1.8.0_502-b07",
            "semver": "8.0.502+7"
        }))
        .unwrap();

        assert_eq!(AdoptiumClient::normalized_version(&java_21), "21.0.2+13");
        assert_eq!(AdoptiumClient::normalized_version(&java_8), "8.0.502+7");
        assert_eq!(
            AdoptiumClient::version_range("21.0.2+13").unwrap(),
            "[21.0.2,21.0.3)"
        );
        assert_eq!(
            AdoptiumClient::version_range("21.0").unwrap(),
            "[21.0,21.1)"
        );
        assert_eq!(AdoptiumClient::version_range("21+13").unwrap(), "[21,22)");
    }

    #[tokio::test]
    async fn major_only_query_uses_latest_shape() {
        let server = MockServer::start().await;
        let filename = "OpenJDK21U-jdk_x64_windows_hotspot_21.0.12_8.zip";

        Mock::given(method("GET"))
            .and(path("/assets/latest/21/hotspot"))
            .and(query_param("os", "windows"))
            .and(query_param("architecture", "x64"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([{
                "binary": binary_json(filename, "https://example.com/latest.zip"),
                "release_name": "jdk-21.0.12+8",
                "version": version_json(21, 0, 12, 8)
            }])))
            .expect(1)
            .mount(&server)
            .await;

        let client = AdoptiumClient::with_base_url(server.uri(), None).unwrap();
        let packages = client.query_packages(&query(None)).await.unwrap();

        assert_eq!(packages.len(), 1);
        assert_eq!(packages[0].java_version, "21.0.12+8");
        assert_eq!(packages[0].filename, filename);
    }

    #[tokio::test]
    async fn historical_query_and_download_use_version_endpoint() {
        let server = MockServer::start().await;
        let release = release_json(21, 0, 2, 13);

        Mock::given(method("GET"))
            .and(path_regex(r"^/assets/version/.*$"))
            .and(query_param("page", "0"))
            .and(query_param("page_size", "20"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([release])))
            .expect(2)
            .mount(&server)
            .await;

        let client = AdoptiumClient::with_base_url(server.uri(), None).unwrap();
        let packages = client
            .query_packages(&query(Some("21.0.2+13")))
            .await
            .unwrap();
        assert_eq!(packages.len(), 1);
        assert_eq!(packages[0].java_version, "21.0.2+13");

        let download = client.resolve_download(&packages[0]).await.unwrap();
        assert_eq!(
            download.url,
            format!("https://example.com/{}", packages[0].filename)
        );
        assert_eq!(
            download.checksum_sha256.as_deref(),
            Some("abcdef1234567890")
        );

        let requests = server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 2);
        for request in requests {
            assert!(
                request
                    .url
                    .as_str()
                    .contains("/assets/version/%5B21.0.2%2C21.0.3%29"),
                "{}",
                request.url
            );
        }
    }

    #[tokio::test]
    async fn historical_query_pages_until_an_exact_build_matches() {
        let server = MockServer::start().await;
        let first_page: Vec<_> = (21..=40)
            .rev()
            .map(|build| release_json(21, 0, 2, build))
            .collect();

        Mock::given(method("GET"))
            .and(path_regex(r"^/assets/version/.*$"))
            .and(query_param("page", "0"))
            .respond_with(ResponseTemplate::new(200).set_body_json(first_page))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path_regex(r"^/assets/version/.*$"))
            .and(query_param("page", "1"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(json!([release_json(21, 0, 2, 13)])),
            )
            .expect(1)
            .mount(&server)
            .await;

        let client = AdoptiumClient::with_base_url(server.uri(), None).unwrap();
        let packages = client
            .query_packages(&query(Some("21.0.2+13")))
            .await
            .unwrap();

        assert_eq!(packages.len(), 1);
        assert_eq!(packages[0].java_version, "21.0.2+13");
    }

    #[tokio::test]
    async fn historical_not_found_is_an_empty_package_set() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path_regex(r"^/assets/version/.*$"))
            .respond_with(ResponseTemplate::new(404))
            .expect(1)
            .mount(&server)
            .await;

        let client = AdoptiumClient::with_base_url(server.uri(), None).unwrap();
        let packages = client
            .query_packages(&query(Some("21.0.99")))
            .await
            .unwrap();

        assert!(packages.is_empty());
    }
}
