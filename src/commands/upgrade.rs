use anyhow::{bail, Result};
use console::style;
use serde::Deserialize;
use std::io::Read;

use crate::output;

const GITHUB_REPO: &str = "Shinnosuke0722/jm";
const MAX_SELF_UPDATE_BINARY_SIZE: u64 = 128 * 1024 * 1024;

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    html_url: String,
    assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
    size: u64,
}

pub async fn run(check_only: bool) -> Result<()> {
    let current_version = env!("CARGO_PKG_VERSION");
    output::print_info(&format!(
        "Current version: {}",
        style(current_version).cyan()
    ));

    // Fetch latest release from GitHub
    output::print_info("Checking for updates...");
    let release = fetch_latest_release().await?;

    let latest_version = release
        .tag_name
        .strip_prefix('v')
        .unwrap_or(&release.tag_name);

    if !is_newer(latest_version, current_version) {
        output::print_success("Already up to date");
        return Ok(());
    }

    output::print_info(&format!(
        "New version available: {} -> {}",
        style(current_version).yellow(),
        style(latest_version).green().bold()
    ));

    if check_only {
        println!("  {}", release.html_url);
        return Ok(());
    }

    // Find the right asset for this platform
    let asset_name = get_platform_asset_name()?;
    let asset = release
        .assets
        .iter()
        .find(|a| a.name == asset_name)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "No release asset found for this platform (expected: {})",
                asset_name
            )
        })?;

    output::print_info(&format!(
        "Downloading {} ({:.1} MB)...",
        asset.name,
        asset.size as f64 / 1_048_576.0
    ));

    // Download to temp file
    let client = reqwest::Client::builder()
        .tls_backend_rustls()
        .user_agent(format!("jm/{}", current_version))
        .build()?;

    let response = client.get(&asset.browser_download_url).send().await?;
    if !response.status().is_success() {
        bail!("Download failed: HTTP {}", response.status());
    }
    let bytes = response.bytes().await?;

    // Extract the binary from the archive
    let binary_bytes = extract_binary_from_archive(&bytes, &asset.name)?;

    // Replace the current binary
    let current_exe = std::env::current_exe()?;
    replace_binary(&current_exe, &binary_bytes)?;

    output::print_success(&format!(
        "Upgraded to {}",
        style(latest_version).green().bold()
    ));

    Ok(())
}

async fn fetch_latest_release() -> Result<GitHubRelease> {
    let url = format!(
        "https://api.github.com/repos/{}/releases/latest",
        GITHUB_REPO
    );
    let client = reqwest::Client::builder()
        .tls_backend_rustls()
        .user_agent(format!("jm/{}", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    let response = client.get(&url).send().await?;
    if response.status() == reqwest::StatusCode::NOT_FOUND {
        bail!("No releases found. This is a development build.");
    }
    if !response.status().is_success() {
        bail!("GitHub API error: HTTP {}", response.status());
    }

    let release = response.json().await?;
    Ok(release)
}

fn get_platform_asset_name() -> Result<String> {
    let (os, ext) = match std::env::consts::OS {
        "linux" => ("linux", "tar.gz"),
        "macos" => ("macos", "tar.gz"),
        "windows" => ("windows", "zip"),
        other => bail!("Unsupported OS for self-update: {}", other),
    };

    let arch = match std::env::consts::ARCH {
        "x86_64" => "x86_64",
        "aarch64" => "aarch64",
        other => bail!("Unsupported arch for self-update: {}", other),
    };

    Ok(format!("jm-{}-{}.{}", os, arch, ext))
}

fn extract_binary_from_archive(data: &[u8], asset_name: &str) -> Result<Vec<u8>> {
    let binary_name = if cfg!(windows) { "jm.exe" } else { "jm" };
    let expected_path = std::path::Path::new(binary_name);

    if asset_name.ends_with(".tar.gz") {
        let decoder = flate2::read::GzDecoder::new(data);
        let mut archive = tar::Archive::new(decoder);
        let mut binary = None;
        for entry in archive.entries()? {
            let mut entry = entry?;
            let path = entry.path()?.into_owned();
            if path == expected_path {
                if !entry.header().entry_type().is_file() {
                    bail!("Archive entry '{}' is not a regular file", binary_name);
                }
                if binary.is_some() {
                    bail!("Archive contains multiple '{}' entries", binary_name);
                }
                let size = entry.size();
                binary = Some(read_update_binary(&mut entry, size, binary_name)?);
            }
        }
        binary.ok_or_else(|| anyhow::anyhow!("Binary '{}' not found in archive", binary_name))
    } else if asset_name.ends_with(".zip") {
        let reader = std::io::Cursor::new(data);
        let mut zip = zip::ZipArchive::new(reader)?;
        let mut binary = None;
        for i in 0..zip.len() {
            let mut file = zip.by_index(i)?;
            if file.name_raw() == binary_name.as_bytes() {
                if file.enclosed_name().as_deref() != Some(expected_path) {
                    bail!("Archive entry '{}' has an unsafe path", binary_name);
                }
                if !file.is_file() || file.encrypted() {
                    bail!("Archive entry '{}' is not a regular file", binary_name);
                }
                if !matches!(
                    file.compression(),
                    zip::CompressionMethod::Stored | zip::CompressionMethod::Deflated
                ) {
                    bail!(
                        "Archive entry '{}' uses unsupported compression",
                        binary_name
                    );
                }
                if binary.is_some() {
                    bail!("Archive contains multiple '{}' entries", binary_name);
                }
                let size = file.size();
                binary = Some(read_update_binary(&mut file, size, binary_name)?);
            }
        }
        binary.ok_or_else(|| anyhow::anyhow!("Binary '{}' not found in archive", binary_name))
    } else {
        bail!("Unknown archive format: {}", asset_name);
    }
}

fn read_update_binary<R: Read>(
    reader: &mut R,
    declared_size: u64,
    binary_name: &str,
) -> Result<Vec<u8>> {
    if declared_size == 0 {
        bail!("Archive entry '{}' is empty", binary_name);
    }
    if declared_size > MAX_SELF_UPDATE_BINARY_SIZE {
        bail!(
            "Archive entry '{}' is too large: {} bytes (limit: {})",
            binary_name,
            declared_size,
            MAX_SELF_UPDATE_BINARY_SIZE
        );
    }

    let capacity = usize::try_from(declared_size)
        .map_err(|_| anyhow::anyhow!("Archive entry '{}' is too large", binary_name))?;
    let mut contents = Vec::with_capacity(capacity);
    reader
        .take(MAX_SELF_UPDATE_BINARY_SIZE + 1)
        .read_to_end(&mut contents)?;

    if contents.len() as u64 > MAX_SELF_UPDATE_BINARY_SIZE {
        bail!("Archive entry '{}' exceeds the size limit", binary_name);
    }
    if contents.len() as u64 != declared_size {
        bail!(
            "Archive entry '{}' size mismatch: expected {}, read {}",
            binary_name,
            declared_size,
            contents.len()
        );
    }

    Ok(contents)
}

fn replace_binary(current_exe: &std::path::Path, new_bytes: &[u8]) -> Result<()> {
    // Write to a temp file next to the current binary, then atomically rename
    let parent = current_exe
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Cannot determine binary directory"))?;
    let temp_path = parent.join(".jm-upgrade-tmp");

    std::fs::write(&temp_path, new_bytes)?;

    // Set executable permission on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&temp_path, std::fs::Permissions::from_mode(0o755))?;
    }

    // On Unix: rename is atomic
    // On Windows: rename the old binary first, then move new one in
    #[cfg(unix)]
    {
        std::fs::rename(&temp_path, current_exe)?;
    }

    #[cfg(windows)]
    {
        let backup = parent.join(".jm-old.exe");
        let _ = std::fs::remove_file(&backup);
        std::fs::rename(current_exe, &backup)?;
        if let Err(e) = std::fs::rename(&temp_path, current_exe) {
            // Rollback
            let _ = std::fs::rename(&backup, current_exe);
            return Err(e.into());
        }
        let _ = std::fs::remove_file(&backup);
    }

    Ok(())
}

/// Simple semver comparison: returns true if `new` is newer than `current`.
fn is_newer(new: &str, current: &str) -> bool {
    let parse = |s: &str| -> (u32, u32, u32) {
        let parts: Vec<&str> = s.split('.').collect();
        (
            parts.first().and_then(|p| p.parse().ok()).unwrap_or(0),
            parts.get(1).and_then(|p| p.parse().ok()).unwrap_or(0),
            parts.get(2).and_then(|p| p.parse().ok()).unwrap_or(0),
        )
    };
    parse(new) > parse(current)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn zip_archive_with_file(path: &str, contents: &[u8]) -> Vec<u8> {
        let cursor = std::io::Cursor::new(Vec::new());
        let mut archive = zip::ZipWriter::new(cursor);
        archive
            .start_file(path, zip::write::SimpleFileOptions::default())
            .unwrap();
        archive.write_all(contents).unwrap();
        archive.finish().unwrap().into_inner()
    }

    fn tar_gz_archive_with_file(path: &str, contents: &[u8]) -> Vec<u8> {
        let encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        let mut archive = tar::Builder::new(encoder);
        let mut header = tar::Header::new_gnu();
        header.set_mode(0o755);
        header.set_size(contents.len() as u64);
        header.set_cksum();
        archive
            .append_data(&mut header, path, std::io::Cursor::new(contents))
            .unwrap();
        archive.into_inner().unwrap().finish().unwrap()
    }

    fn tar_gz_archive_with_symlink(path: &str) -> Vec<u8> {
        let encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        let mut archive = tar::Builder::new(encoder);
        let mut header = tar::Header::new_gnu();
        header.set_mode(0o755);
        header.set_size(0);
        header.set_entry_type(tar::EntryType::Symlink);
        archive
            .append_link(&mut header, path, "other-binary")
            .unwrap();
        archive.into_inner().unwrap().finish().unwrap()
    }

    #[test]
    fn version_comparison() {
        assert!(is_newer("0.2.0", "0.1.0"));
        assert!(is_newer("1.0.0", "0.9.9"));
        assert!(is_newer("0.1.1", "0.1.0"));
        assert!(!is_newer("0.1.0", "0.1.0"));
        assert!(!is_newer("0.0.9", "0.1.0"));
    }

    #[test]
    fn platform_asset_name() {
        let name = get_platform_asset_name().unwrap();
        assert!(name.starts_with("jm-"));
        assert!(name.contains("x86_64") || name.contains("aarch64"));
    }

    #[test]
    fn extracts_exact_root_binary_from_zip() {
        let binary_name = if cfg!(windows) { "jm.exe" } else { "jm" };
        let archive = zip_archive_with_file(binary_name, b"valid binary");

        let extracted = extract_binary_from_archive(&archive, "jm-windows-x86_64.zip").unwrap();

        assert_eq!(extracted, b"valid binary");
    }

    #[test]
    fn rejects_zip_entry_that_only_ends_with_binary_name() {
        let binary_name = if cfg!(windows) { "jm.exe" } else { "jm" };
        let archive = zip_archive_with_file(&format!("evil{binary_name}"), b"wrong binary");

        let error = extract_binary_from_archive(&archive, "jm-windows-x86_64.zip").unwrap_err();

        assert!(error.to_string().contains("not found"));
    }

    #[test]
    fn rejects_zip_entry_that_normalizes_to_binary_name() {
        let binary_name = if cfg!(windows) { "jm.exe" } else { "jm" };
        let archive =
            zip_archive_with_file(&format!("directory/../{binary_name}"), b"wrong binary");

        let error = extract_binary_from_archive(&archive, "jm-windows-x86_64.zip").unwrap_err();

        assert!(error.to_string().contains("not found"));
    }

    #[test]
    fn rejects_empty_update_binary() {
        let binary_name = if cfg!(windows) { "jm.exe" } else { "jm" };
        let archive = zip_archive_with_file(binary_name, b"");

        let error = extract_binary_from_archive(&archive, "jm-windows-x86_64.zip").unwrap_err();

        assert!(error.to_string().contains("is empty"));
    }

    #[test]
    fn extracts_exact_root_binary_from_tar_gz() {
        let binary_name = if cfg!(windows) { "jm.exe" } else { "jm" };
        let archive = tar_gz_archive_with_file(binary_name, b"valid binary");

        let extracted = extract_binary_from_archive(&archive, "jm-linux-x86_64.tar.gz").unwrap();

        assert_eq!(extracted, b"valid binary");
    }

    #[test]
    fn rejects_tar_symlink_as_update_binary() {
        let binary_name = if cfg!(windows) { "jm.exe" } else { "jm" };
        let archive = tar_gz_archive_with_symlink(binary_name);

        let error = extract_binary_from_archive(&archive, "jm-linux-x86_64.tar.gz").unwrap_err();

        assert!(error.to_string().contains("not a regular file"));
    }

    #[test]
    fn rejects_declared_update_binary_over_size_limit() {
        let mut reader = std::io::empty();
        let error =
            read_update_binary(&mut reader, MAX_SELF_UPDATE_BINARY_SIZE + 1, "jm").unwrap_err();

        assert!(error.to_string().contains("is too large"));
    }
}
