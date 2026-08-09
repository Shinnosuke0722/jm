use indicatif::{ProgressBar, ProgressStyle};
use jm_api::models::DownloadInfo;
use jm_core::error::{JmError, Result};
use std::path::{Component, Path, PathBuf};
use tokio::io::AsyncWriteExt;

/// Return whether `value` is a single, ordinary filename component on every
/// supported platform.
///
/// Both separator styles and all other Win32-forbidden filename characters are
/// rejected explicitly so that a filename accepted on Unix is also safe on
/// Windows. Windows device names and trailing spaces or periods are rejected as
/// well, regardless of the host platform.
pub fn is_safe_filename_component(value: &str) -> bool {
    if value.is_empty()
        || value.ends_with(' ')
        || value.ends_with('.')
        || value.chars().any(|c| {
            matches!(
                c,
                '/' | '\\' | ':' | '<' | '>' | '"' | '|' | '?' | '*' | '\0'
            ) || c.is_control()
        })
        || is_windows_reserved_device_name(value)
    {
        return false;
    }

    let mut components = Path::new(value).components();
    matches!(
        (components.next(), components.next()),
        (Some(Component::Normal(_)), None)
    )
}

fn is_windows_reserved_device_name(value: &str) -> bool {
    // Win32 reserves these basenames even when an extension is present, for
    // example `NUL.zip` and `COM1.tar.gz`.
    let basename = value.split('.').next().unwrap_or(value);

    if ["CON", "PRN", "AUX", "NUL"]
        .iter()
        .any(|reserved| basename.eq_ignore_ascii_case(reserved))
    {
        return true;
    }

    let (Some(prefix), Some(number)) = (basename.get(..3), basename.get(3..)) else {
        return false;
    };

    (prefix.eq_ignore_ascii_case("COM") || prefix.eq_ignore_ascii_case("LPT"))
        && matches!(
            number,
            "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"
        )
}

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
    if !is_safe_filename_component(&info.filename) {
        return Err(JmError::DownloadFailed(format!(
            "unsafe download filename {:?}: expected one ordinary filename component",
            info.filename
        )));
    }

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

    let mut builder = reqwest::Client::builder()
        .tls_backend_rustls()
        .user_agent(format!("jm/{}", env!("CARGO_PKG_VERSION")));

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_regular_archive_filenames() {
        for filename in [
            "OpenJDK21U-jdk_x64_windows_hotspot_21.0.8_9.zip",
            "OpenJDK21U-jdk_x64_linux_hotspot_21.0.8_9.tar.gz",
            "zulu21.44.17-ca-jdk21.0.8-linux_x64.tar.gz",
            "console.zip",
            "null.zip",
            "COM0.zip",
            "COM10.zip",
            "LPT0.zip",
            "LPT10.zip",
            ".jdk.zip",
        ] {
            assert!(is_safe_filename_component(filename), "{filename}");
        }
    }

    #[test]
    fn rejects_cross_platform_path_escape_filenames() {
        for filename in [
            "",
            ".",
            "..",
            "../jdk.zip",
            "..\\jdk.zip",
            "nested/jdk.zip",
            "nested\\jdk.zip",
            "/tmp/jdk.zip",
            "C:\\temp\\jdk.zip",
            "C:/temp/jdk.zip",
            "C:jdk.zip",
            "\\\\server\\share\\jdk.zip",
            "jdk.zip:stream",
            "jdk<old>.zip",
            "jdk>new.zip",
            "jdk\"quoted.zip",
            "jdk|pipe.zip",
            "jdk?.zip",
            "jdk*.zip",
            "jdk.zip ",
            "jdk.zip.",
            "jdk\n.zip",
            "jdk\0.zip",
        ] {
            assert!(!is_safe_filename_component(filename), "{filename:?}");
        }
    }

    #[test]
    fn rejects_windows_reserved_device_basenames() {
        for filename in [
            "CON",
            "con.zip",
            "PrN.tar.gz",
            "AUX.zip",
            "nul.zip",
            "COM1.zip",
            "com9.tar.gz",
            "LPT1.zip",
            "lPt9.tar.gz",
            "COM¹.zip",
            "com².tar.gz",
            "COM³.zip",
            "LPT¹.zip",
            "lpt².tar.gz",
            "LPT³.zip",
        ] {
            assert!(!is_safe_filename_component(filename), "{filename:?}");
        }
    }
}
