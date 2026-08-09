use jm_api::cache::ApiCache;
use jm_api::disco::DiscoClient;
use jm_api::provider::{JdkProvider, PackageQuery};
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn test_cache() -> (tempfile::TempDir, ApiCache) {
    let dir = tempfile::tempdir().unwrap();
    let cache = ApiCache::new(dir.path().to_path_buf());
    (dir, cache)
}

fn sample_packages_response() -> serde_json::Value {
    serde_json::json!({
        "result": [{
            "id": "abc123",
            "archive_type": "tar.gz",
            "distribution": "temurin",
            "major_version": 21,
            "java_version": "21.0.2+13",
            "jdk_version": 21,
            "release_status": "ga",
            "term_of_support": "LTS",
            "operating_system": "macos",
            "architecture": "aarch64",
            "package_type": "jdk",
            "directly_downloadable": true,
            "filename": "OpenJDK21U-jdk_aarch64_mac_hotspot_21.0.2_13.tar.gz",
            "links": {
                "pkg_info_uri": "https://api.foojay.io/disco/v3.0/ids/abc123",
                "pkg_download_redirect": ""
            },
            "size": 198000000
        }],
        "message": "1 package(s) found"
    })
}

fn sample_package_info_response() -> serde_json::Value {
    serde_json::json!({
        "result": [{
            "filename": "OpenJDK21U-jdk_aarch64_mac_hotspot_21.0.2_13.tar.gz",
            "direct_download_uri": "https://example.com/download.tar.gz",
            "checksum": "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
            "checksum_type": "sha256"
        }],
        "message": ""
    })
}

#[tokio::test]
async fn query_packages_success() {
    let server = MockServer::start().await;
    let (_dir, cache) = test_cache();

    Mock::given(method("GET"))
        .and(path("/packages"))
        .and(query_param("jdk_version", "21"))
        .and(query_param("distribution", "temurin"))
        .respond_with(ResponseTemplate::new(200).set_body_json(sample_packages_response()))
        .mount(&server)
        .await;

    let client = DiscoClient::new(server.uri(), cache).unwrap();
    let query = PackageQuery {
        major_version: Some(21),
        distribution: Some("temurin".into()),
        ..Default::default()
    };

    let packages = client.query_packages(&query).await.unwrap();
    assert_eq!(packages.len(), 1);
    assert_eq!(packages[0].java_version, "21.0.2+13");
    assert_eq!(packages[0].major_version, 21);
    assert_eq!(packages[0].distribution, "temurin");
    assert_eq!(packages[0].term_of_support, "LTS");
}

#[tokio::test]
async fn query_packages_empty_result() {
    let server = MockServer::start().await;
    let (_dir, cache) = test_cache();

    Mock::given(method("GET"))
        .and(path("/packages"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"result": [], "message": "0 packages"})),
        )
        .mount(&server)
        .await;

    let client = DiscoClient::new(server.uri(), cache).unwrap();
    let query = PackageQuery {
        major_version: Some(99),
        ..Default::default()
    };

    let packages = client.query_packages(&query).await.unwrap();
    assert!(packages.is_empty());
}

#[tokio::test]
async fn query_packages_http_error() {
    let server = MockServer::start().await;
    let (_dir, cache) = test_cache();

    Mock::given(method("GET"))
        .and(path("/packages"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
        .mount(&server)
        .await;

    let client = DiscoClient::new(server.uri(), cache).unwrap();
    let query = PackageQuery {
        major_version: Some(21),
        ..Default::default()
    };

    let result = client.query_packages(&query).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("500"),
        "Error should mention status code: {err}"
    );
}

#[tokio::test]
async fn resolve_download_success() {
    let server = MockServer::start().await;
    let (_dir, cache) = test_cache();

    Mock::given(method("GET"))
        .and(path("/packages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(sample_packages_response()))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/ids/abc123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(sample_package_info_response()))
        .mount(&server)
        .await;

    let client = DiscoClient::new(server.uri(), cache).unwrap();
    let query = PackageQuery {
        major_version: Some(21),
        distribution: Some("temurin".into()),
        ..Default::default()
    };

    let packages = client.query_packages(&query).await.unwrap();
    let download = client.resolve_download(&packages[0]).await.unwrap();

    assert_eq!(download.url, "https://example.com/download.tar.gz");
    assert!(download.checksum_sha256.is_some());
    assert_eq!(
        download.checksum_sha256.unwrap(),
        "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
    );
}

#[tokio::test]
async fn cache_prevents_duplicate_requests() {
    let server = MockServer::start().await;
    let (_dir, cache) = test_cache();

    Mock::given(method("GET"))
        .and(path("/packages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(sample_packages_response()))
        .expect(1) // Must be called exactly once
        .mount(&server)
        .await;

    let client = DiscoClient::new(server.uri(), cache).unwrap();
    let query = PackageQuery {
        major_version: Some(21),
        distribution: Some("temurin".into()),
        ..Default::default()
    };

    // First call hits the server
    let p1 = client.query_packages(&query).await.unwrap();
    // Second call should use cache
    let p2 = client.query_packages(&query).await.unwrap();

    assert_eq!(p1.len(), p2.len());
    assert_eq!(p1[0].java_version, p2[0].java_version);
    // wiremock will verify expect(1) on drop
}

#[tokio::test]
async fn list_distributions_success() {
    let server = MockServer::start().await;
    let (_dir, cache) = test_cache();

    Mock::given(method("GET"))
        .and(path("/distributions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "result": [
                {
                    "name": "Temurin",
                    "api_parameter": "temurin",
                    "maintained": true,
                    "available": true,
                    "build_of_openjdk": true,
                    "build_of_graalvm": false,
                    "official_uri": "https://adoptium.net"
                }
            ],
            "message": "1 distribution(s)"
        })))
        .mount(&server)
        .await;

    let client = DiscoClient::new(server.uri(), cache).unwrap();
    let distros = client.list_distributions().await.unwrap();
    assert_eq!(distros.len(), 1);
    assert_eq!(distros[0].api_parameter, "temurin");
    assert!(distros[0].maintained);
}

#[tokio::test]
async fn malformed_json_returns_error() {
    let server = MockServer::start().await;
    let (_dir, cache) = test_cache();

    Mock::given(method("GET"))
        .and(path("/packages"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
        .mount(&server)
        .await;

    let client = DiscoClient::new(server.uri(), cache).unwrap();
    let query = PackageQuery {
        major_version: Some(21),
        ..Default::default()
    };

    let result = client.query_packages(&query).await;
    assert!(result.is_err());
}
