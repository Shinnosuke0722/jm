use jm_core::error::{JmError, Result};
use jm_core::platform::Os;
use std::path::{Path, PathBuf};

/// Extract a JDK archive and return the path to the JDK home directory.
///
/// Handles the fact that different distributions have different archive structures:
/// - Temurin: `jdk-21.0.2+13/bin/java`
/// - Zulu: `zulu21.34.19-ca-jdk21.0.3-macosx_aarch64/bin/java`
/// - macOS: `jdk-21.0.2+13/Contents/Home/bin/java`
///
/// The returned path is the directory containing `bin/java`.
pub fn extract_archive(archive_path: &Path, dest_dir: &Path, os: Os) -> Result<PathBuf> {
    std::fs::create_dir_all(dest_dir)?;

    let ext = archive_path.to_str().unwrap_or("");

    if ext.ends_with(".tar.gz") || ext.ends_with(".tgz") {
        extract_tar_gz(archive_path, dest_dir)?;
    } else if ext.ends_with(".zip") {
        extract_zip(archive_path, dest_dir)?;
    } else {
        return Err(JmError::ExtractionFailed(format!(
            "unsupported archive format: {}",
            archive_path.display()
        )));
    }

    // Find the JDK home directory (the one containing bin/java)
    find_jdk_home(dest_dir, os)
}

fn extract_tar_gz(archive: &Path, dest: &Path) -> Result<()> {
    let file = std::fs::File::open(archive)?;
    let decoder = flate2::read::GzDecoder::new(file);
    let mut tar = tar::Archive::new(decoder);
    tar.unpack(dest)
        .map_err(|e| JmError::ExtractionFailed(e.to_string()))?;
    Ok(())
}

fn extract_zip(archive: &Path, dest: &Path) -> Result<()> {
    let file = std::fs::File::open(archive)?;
    let mut zip =
        zip::ZipArchive::new(file).map_err(|e| JmError::ExtractionFailed(e.to_string()))?;
    zip.extract(dest)
        .map_err(|e| JmError::ExtractionFailed(e.to_string()))?;
    Ok(())
}

/// Scan `dir` for the JDK home directory (containing `bin/java` or `bin/java.exe`).
fn find_jdk_home(dir: &Path, os: Os) -> Result<PathBuf> {
    let binary_name = os.java_binary_name();

    // Check common patterns:
    // 1. dir/*/bin/java (standard layout)
    // 2. dir/*/Contents/Home/bin/java (macOS .tar.gz from some distributions)

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            // Direct: path/bin/java
            if path.join("bin").join(binary_name).exists() {
                return Ok(path);
            }

            // macOS bundle: path/Contents/Home/bin/java
            let macos_home = path.join("Contents").join("Home");
            if macos_home.join("bin").join(binary_name).exists() {
                return Ok(macos_home);
            }
        }
    }

    Err(JmError::JdkHomeNotFound)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_jdk_home_standard_layout() {
        let dir = tempfile::tempdir().unwrap();
        let jdk_dir = dir.path().join("jdk-21.0.2+13");
        std::fs::create_dir_all(jdk_dir.join("bin")).unwrap();

        #[cfg(unix)]
        std::fs::write(jdk_dir.join("bin/java"), "").unwrap();
        #[cfg(windows)]
        std::fs::write(jdk_dir.join("bin/java.exe"), "").unwrap();

        let os = if cfg!(windows) {
            Os::Windows
        } else {
            Os::Linux
        };
        let home = find_jdk_home(dir.path(), os).unwrap();
        assert_eq!(home, jdk_dir);
    }

    #[test]
    fn find_jdk_home_macos_bundle() {
        let dir = tempfile::tempdir().unwrap();
        let bundle_dir = dir
            .path()
            .join("jdk-21.0.2+13")
            .join("Contents")
            .join("Home");
        std::fs::create_dir_all(bundle_dir.join("bin")).unwrap();

        std::fs::write(bundle_dir.join("bin/java"), "").unwrap();

        let home = find_jdk_home(dir.path(), Os::MacOs).unwrap();
        assert_eq!(home, bundle_dir);
    }
}
