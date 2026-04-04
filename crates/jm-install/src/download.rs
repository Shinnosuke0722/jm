use indicatif::{ProgressBar, ProgressStyle};
use jm_api::models::DownloadInfo;
use jm_core::error::{JmError, Result};
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;

/// Guard that cleans up the .part file if dropped before `defuse()` is called.
/// Ensures partial downloads are removed on error, panic, or Ctrl+C.
struct PartFileGuard {
    path: PathBuf,
    armed: bool,
}

impl PartFileGuard {
    fn new(path: PathBuf) -> Self {
        Self { path, armed: true }
    }

    /// Disarm the guard — the .part file has been successfully renamed.
    fn defuse(&mut self) {
        self.armed = false;
    }
}

impl Drop for PartFileGuard {
    fn drop(&mut self) {
        if self.armed {
            let _ = std::fs::remove_file(&self.path);
        }
    }
}

/// Download a JDK archive with progress reporting.
///
/// Returns the path to the downloaded file.
pub async fn download_jdk(info: &DownloadInfo, downloads_dir: &Path) -> Result<PathBuf> {
    download_jdk_with_proxy(info, downloads_dir, None).await
}

/// Download a JDK archive with progress reporting and optional HTTP proxy.
pub async fn download_jdk_with_proxy(
    info: &DownloadInfo,
    downloads_dir: &Path,
    proxy: Option<&str>,
) -> Result<PathBuf> {
    std::fs::create_dir_all(downloads_dir)?;
    let dest = downloads_dir.join(&info.filename);

    // If already downloaded and verified, skip
    if dest.exists() {
        return Ok(dest);
    }

    // Clean up stale .part files from previous interrupted downloads
    let tmp_path = dest.with_extension("part");
    if tmp_path.exists() {
        let _ = std::fs::remove_file(&tmp_path);
    }

    let mut builder =
        reqwest::Client::builder().user_agent(format!("jm/{}", env!("CARGO_PKG_VERSION")));

    if let Some(proxy_url) = proxy {
        builder = builder.proxy(
            reqwest::Proxy::all(proxy_url)
                .map_err(|e| JmError::DownloadFailed(format!("invalid proxy URL: {}", e)))?,
        );
    }

    let client = builder
        .build()
        .map_err(|e| JmError::DownloadFailed(e.to_string()))?;

    let response = client
        .get(&info.url)
        .send()
        .await
        .map_err(|e| JmError::DownloadFailed(e.to_string()))?;

    if !response.status().is_success() {
        return Err(JmError::DownloadFailed(format!(
            "HTTP {}",
            response.status()
        )));
    }

    let total_size = response.content_length().unwrap_or(info.size);

    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .unwrap()
            .progress_chars("#>-"),
    );
    pb.set_message(info.filename.clone());

    let mut guard = PartFileGuard::new(tmp_path.clone());
    let mut file = tokio::fs::File::create(&tmp_path)
        .await
        .map_err(|e| JmError::DownloadFailed(e.to_string()))?;

    let mut stream = response.bytes_stream();
    use futures_util::StreamExt;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| JmError::DownloadFailed(e.to_string()))?;
        file.write_all(&chunk)
            .await
            .map_err(|e| JmError::DownloadFailed(e.to_string()))?;
        pb.inc(chunk.len() as u64);
    }

    file.flush()
        .await
        .map_err(|e| JmError::DownloadFailed(e.to_string()))?;
    drop(file);

    // Atomic rename — only after full download succeeds
    std::fs::rename(&tmp_path, &dest)?;
    guard.defuse();
    pb.finish_with_message("Download complete");

    Ok(dest)
}
