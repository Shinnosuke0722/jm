use crate::cache::ApiCache;
use crate::models::*;
use crate::provider::{JdkProvider, PackageQuery};
use jm_core::error::JmError;
use reqwest::Client;

/// Client for the Foojay Disco API v3.
pub struct DiscoClient {
    client: Client,
    base_url: String,
    cache: ApiCache,
}

impl DiscoClient {
    pub fn new(base_url: String, cache: ApiCache) -> Result<Self, JmError> {
        Self::with_proxy(base_url, cache, None)
    }

    pub fn with_proxy(
        base_url: String,
        cache: ApiCache,
        proxy: Option<&str>,
    ) -> Result<Self, JmError> {
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
            base_url,
            cache,
        })
    }

    /// List all available distributions.
    pub async fn list_distributions(&self) -> Result<Vec<DiscoDistribution>, JmError> {
        let url = format!("{}/distributions", self.base_url);
        let body = self.cached_get(&url, "distributions", 24 * 3600).await?;
        let resp: DiscoResponse<DiscoDistribution> =
            serde_json::from_str(&body).map_err(|e| JmError::ApiError(e.to_string()))?;
        Ok(resp.result)
    }

    /// List all major Java versions.
    pub async fn list_major_versions(&self) -> Result<Vec<DiscoMajorVersion>, JmError> {
        let url = format!("{}/major_versions", self.base_url);
        let body = self.cached_get(&url, "major_versions", 6 * 3600).await?;
        let resp: DiscoResponse<DiscoMajorVersion> =
            serde_json::from_str(&body).map_err(|e| JmError::ApiError(e.to_string()))?;
        Ok(resp.result)
    }

    /// Query packages with filters.
    pub async fn query_packages_raw(
        &self,
        query: &PackageQuery,
    ) -> Result<Vec<DiscoPackage>, JmError> {
        let mut url = format!("{}/packages", self.base_url);
        let mut params = Vec::new();

        if let Some(v) = query.major_version {
            params.push(format!("jdk_version={}", v));
        }
        if let Some(ref d) = query.distribution {
            params.push(format!("distribution={}", d));
        }
        if let Some(ref os) = query.operating_system {
            params.push(format!("operating_system={}", os));
        }
        if let Some(ref arch) = query.architecture {
            params.push(format!("architecture={}", arch));
        }
        if let Some(ref at) = query.archive_type {
            params.push(format!("archive_type={}", at));
        }
        if let Some(ref pt) = query.package_type {
            params.push(format!("package_type={}", pt));
        }
        if let Some(ref rs) = query.release_status {
            params.push(format!("release_status={}", rs));
        }
        if let Some(ref l) = query.latest {
            params.push(format!("latest={}", l));
        }

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        let cache_key = format!("packages_{}", params.join("_"));
        let body = self.cached_get(&url, &cache_key, 3600).await?;
        let resp: DiscoResponse<DiscoPackage> =
            serde_json::from_str(&body).map_err(|e| JmError::ApiError(e.to_string()))?;
        Ok(resp.result)
    }

    /// Get detailed package info (download URL, checksum) by package id.
    pub async fn get_package_info(&self, id: &str) -> Result<DiscoPackageInfo, JmError> {
        let url = format!("{}/ids/{}", self.base_url, id);
        let cache_key = format!("ids_{}", id);
        let body = self.cached_get(&url, &cache_key, 7 * 24 * 3600).await?;
        let resp: DiscoResponse<DiscoPackageInfo> =
            serde_json::from_str(&body).map_err(|e| JmError::ApiError(e.to_string()))?;
        resp.result
            .into_iter()
            .next()
            .ok_or_else(|| JmError::ApiError(format!("no package info found for id: {}", id)))
    }

    async fn cached_get(
        &self,
        url: &str,
        cache_key: &str,
        ttl_secs: u64,
    ) -> Result<String, JmError> {
        // Try cache first
        if let Some(cached) = self.cache.get(cache_key, ttl_secs) {
            return Ok(cached);
        }

        // Fetch from API
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| JmError::ApiError(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(JmError::ApiError(format!(
                "API returned status {}: {}",
                response.status(),
                url
            )));
        }

        let body = response
            .text()
            .await
            .map_err(|e| JmError::ApiError(format!("failed to read response body: {}", e)))?;

        // Cache the response
        let _ = self.cache.put(cache_key, &body);

        Ok(body)
    }
}

#[async_trait::async_trait]
impl JdkProvider for DiscoClient {
    fn name(&self) -> &str {
        "Foojay Disco"
    }

    async fn query_packages(&self, query: &PackageQuery) -> Result<Vec<JdkPackage>, JmError> {
        let packages = self.query_packages_raw(query).await?;
        Ok(packages
            .into_iter()
            .map(|p| JdkPackage {
                id: p.id,
                distribution: p.distribution,
                major_version: p.major_version,
                java_version: p.java_version,
                operating_system: p.operating_system,
                architecture: p.architecture,
                archive_type: p.archive_type,
                filename: p.filename,
                term_of_support: p.term_of_support,
                directly_downloadable: p.directly_downloadable,
                size: p.size,
            })
            .collect())
    }

    async fn resolve_download(&self, package: &JdkPackage) -> Result<DownloadInfo, JmError> {
        let info = self.get_package_info(&package.id).await?;
        let checksum = if info.checksum_type.to_lowercase() == "sha256" && !info.checksum.is_empty()
        {
            Some(info.checksum)
        } else {
            None
        };
        Ok(DownloadInfo {
            url: info.direct_download_uri,
            checksum_sha256: checksum,
            filename: info.filename,
            size: package.size,
        })
    }
}
