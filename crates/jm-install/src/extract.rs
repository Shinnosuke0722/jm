use crate::download::is_safe_filename_component;
use jm_core::error::{JmError, Result};
use jm_core::platform::Os;
use std::{
    collections::HashSet,
    fs::OpenOptions,
    io::{self, Read, Seek},
    path::{Component, Path, PathBuf},
};

const MAX_ZIP_ENTRIES: usize = 100_000;
const MAX_ZIP_ENTRY_SIZE: u64 = 2 * 1024 * 1024 * 1024;
const MAX_ZIP_TOTAL_SIZE: u64 = 4 * 1024 * 1024 * 1024;

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
    validate_zip_archive(&mut zip)?;
    extract_validated_zip(&mut zip, dest)
}

fn validate_zip_archive<R: Read + Seek>(archive: &mut zip::ZipArchive<R>) -> Result<()> {
    validate_zip_archive_with_limits(
        archive,
        MAX_ZIP_ENTRIES,
        MAX_ZIP_ENTRY_SIZE,
        MAX_ZIP_TOTAL_SIZE,
    )
}

fn validate_zip_archive_with_limits<R: Read + Seek>(
    archive: &mut zip::ZipArchive<R>,
    max_entries: usize,
    max_entry_size: u64,
    max_total_size: u64,
) -> Result<()> {
    if archive.len() > max_entries {
        return Err(JmError::ExtractionFailed(format!(
            "ZIP archive contains too many entries: {} (limit: {max_entries})",
            archive.len()
        )));
    }

    if archive
        .has_overlapping_files()
        .map_err(|e| JmError::ExtractionFailed(e.to_string()))?
    {
        return Err(JmError::ExtractionFailed(
            "ZIP archive contains overlapping file data".to_string(),
        ));
    }

    let mut paths = HashSet::with_capacity(archive.len());
    let mut total_size = 0_u64;
    for index in 0..archive.len() {
        let file = archive
            .by_index(index)
            .map_err(|e| JmError::ExtractionFailed(e.to_string()))?;
        let name = file.name().to_string();
        let path = file.enclosed_name().ok_or_else(|| {
            JmError::ExtractionFailed(format!("unsafe path in ZIP archive: {name}"))
        })?;

        if file.size() > max_entry_size {
            return Err(JmError::ExtractionFailed(format!(
                "ZIP entry is too large: {name} ({} bytes, limit: {max_entry_size})",
                file.size()
            )));
        }
        total_size = total_size
            .checked_add(file.size())
            .ok_or_else(|| JmError::ExtractionFailed("ZIP archive size overflow".to_string()))?;
        if total_size > max_total_size {
            return Err(JmError::ExtractionFailed(format!(
                "ZIP archive expands beyond the {max_total_size}-byte limit"
            )));
        }

        if file.encrypted() {
            return Err(JmError::ExtractionFailed(format!(
                "encrypted ZIP entry is not supported: {name}"
            )));
        }
        if file.is_symlink() {
            return Err(JmError::ExtractionFailed(format!(
                "symbolic links are not allowed in ZIP archives: {name}"
            )));
        }
        if file.is_dir() && file.size() != 0 {
            return Err(JmError::ExtractionFailed(format!(
                "ZIP directory entry contains file data: {name}"
            )));
        }
        if !matches!(
            file.compression(),
            zip::CompressionMethod::Stored | zip::CompressionMethod::Deflated
        ) {
            return Err(JmError::ExtractionFailed(format!(
                "unsupported ZIP compression method for entry: {name}"
            )));
        }
        let path_key = portable_zip_path_key(&path, &name)?;
        if !paths.insert(path_key) {
            return Err(JmError::ExtractionFailed(format!(
                "duplicate ZIP entry path: {}",
                path.display()
            )));
        }
    }

    Ok(())
}

fn extract_validated_zip<R: Read + Seek>(
    archive: &mut zip::ZipArchive<R>,
    dest: &Path,
) -> Result<()> {
    if std::fs::read_dir(dest)?.next().is_some() {
        return Err(JmError::ExtractionFailed(
            "ZIP extraction destination must be empty".to_string(),
        ));
    }

    let mut total_written = 0_u64;
    #[cfg(unix)]
    let mut unix_modes = Vec::new();

    for index in 0..archive.len() {
        let mut file = archive
            .by_index(index)
            .map_err(|e| JmError::ExtractionFailed(e.to_string()))?;
        let name = file.name().to_string();
        let relative_path = file.enclosed_name().ok_or_else(|| {
            JmError::ExtractionFailed(format!("unsafe path in ZIP archive: {name}"))
        })?;
        let output_path = dest.join(relative_path);

        if file.is_dir() {
            let mut extra = [0_u8; 1];
            let extra_bytes = file
                .read(&mut extra)
                .map_err(|e| JmError::ExtractionFailed(e.to_string()))?;
            if extra_bytes != 0 {
                return Err(JmError::ExtractionFailed(format!(
                    "ZIP directory entry expands beyond its declared size: {name}"
                )));
            }
            std::fs::create_dir_all(&output_path)?;
            #[cfg(unix)]
            if let Some(mode) = file.unix_mode() {
                // A trailing slash is authoritative for ZIP directories, but
                // some writers still encode them with regular-file mode bits.
                // Keep the archived group/world bits while ensuring the owner
                // can traverse and populate the extracted directory.
                unix_modes.push((output_path, (mode & 0o777) | 0o700));
            }
            continue;
        }

        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut output = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&output_path)?;

        let declared_size = file.size();
        let written = {
            let mut limited = (&mut file).take(declared_size);
            io::copy(&mut limited, &mut output)
                .map_err(|e| JmError::ExtractionFailed(e.to_string()))?
        };
        if written != declared_size {
            return Err(JmError::ExtractionFailed(format!(
                "ZIP entry size mismatch for {name}: expected {declared_size}, wrote {written}"
            )));
        }

        // Reading once beyond the declared size both detects over-expansion and
        // forces the ZIP reader to validate the entry's CRC at EOF.
        let mut extra = [0_u8; 1];
        let extra_bytes = file
            .read(&mut extra)
            .map_err(|e| JmError::ExtractionFailed(e.to_string()))?;
        if extra_bytes != 0 {
            return Err(JmError::ExtractionFailed(format!(
                "ZIP entry expands beyond its declared size: {name}"
            )));
        }

        total_written = total_written
            .checked_add(written)
            .ok_or_else(|| JmError::ExtractionFailed("ZIP archive size overflow".to_string()))?;
        if total_written > MAX_ZIP_TOTAL_SIZE {
            return Err(JmError::ExtractionFailed(format!(
                "ZIP archive expands beyond the {MAX_ZIP_TOTAL_SIZE}-byte limit"
            )));
        }

        #[cfg(unix)]
        if let Some(mode) = file.unix_mode() {
            unix_modes.push((output_path, mode & 0o777));
        }
    }

    #[cfg(unix)]
    {
        use std::cmp::Reverse;
        use std::os::unix::fs::PermissionsExt;

        unix_modes.sort_by_key(|(path, _)| Reverse(path.components().count()));
        for (path, mode) in unix_modes {
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))?;
        }
    }

    Ok(())
}

fn portable_zip_path_key(path: &Path, original_name: &str) -> Result<String> {
    let mut normalized = String::new();

    for component in path.components() {
        let Component::Normal(component) = component else {
            return Err(JmError::ExtractionFailed(format!(
                "unsafe path in ZIP archive: {original_name}"
            )));
        };
        let component = component.to_str().ok_or_else(|| {
            JmError::ExtractionFailed(format!("non-UTF-8 path in ZIP archive: {original_name}"))
        })?;
        if !is_safe_filename_component(component) {
            return Err(JmError::ExtractionFailed(format!(
                "non-portable path in ZIP archive: {original_name}"
            )));
        }

        if !normalized.is_empty() {
            normalized.push('/');
        }
        normalized.push_str(component);
    }

    let raw_path = original_name.strip_suffix('/').unwrap_or(original_name);
    if normalized.is_empty() || raw_path != normalized {
        return Err(JmError::ExtractionFailed(format!(
            "non-canonical path in ZIP archive: {original_name}"
        )));
    }

    Ok(normalized.to_lowercase())
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
    use std::io::Write;

    #[test]
    fn extract_archive_reads_deflated_zip() {
        let dir = tempfile::tempdir().unwrap();
        let archive_path = dir.path().join("jdk.zip");
        let binary_name = if cfg!(windows) { "java.exe" } else { "java" };

        {
            let file = std::fs::File::create(&archive_path).unwrap();
            let mut archive = zip::ZipWriter::new(file);
            let options = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);
            archive
                .start_file(format!("jdk-21/bin/{binary_name}"), options)
                .unwrap();
            archive.write_all(b"test java binary").unwrap();
            archive.finish().unwrap();
        }

        let os = if cfg!(windows) {
            Os::Windows
        } else {
            Os::Linux
        };
        let destination = dir.path().join("extracted");
        let home = extract_archive(&archive_path, &destination, os).unwrap();

        assert_eq!(home, destination.join("jdk-21"));
        assert!(home.join("bin").join(binary_name).is_file());
    }

    #[test]
    fn extract_archive_accepts_deflated_empty_directory_entry() {
        let dir = tempfile::tempdir().unwrap();
        let archive_path = dir.path().join("jdk.zip");
        let binary_name = if cfg!(windows) { "java.exe" } else { "java" };

        {
            let file = std::fs::File::create(&archive_path).unwrap();
            let mut archive = zip::ZipWriter::new(file);
            let options = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);
            archive
                .start_file("jdk-21/", options.unix_permissions(0o644))
                .unwrap();
            archive
                .start_file(format!("jdk-21/bin/{binary_name}"), options)
                .unwrap();
            archive.write_all(b"test java binary").unwrap();
            archive.finish().unwrap();
        }

        let os = if cfg!(windows) {
            Os::Windows
        } else {
            Os::Linux
        };
        let destination = dir.path().join("extracted");
        let home = extract_archive(&archive_path, &destination, os).unwrap();

        assert_eq!(home, destination.join("jdk-21"));
        assert!(home.join("bin").join(binary_name).is_file());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let mode = std::fs::metadata(&home).unwrap().permissions().mode();
            assert_eq!(mode & 0o700, 0o700);
        }
    }

    #[test]
    fn extract_archive_reads_stored_zip() {
        let dir = tempfile::tempdir().unwrap();
        let archive_path = dir.path().join("jdk.zip");
        let binary_name = if cfg!(windows) { "java.exe" } else { "java" };

        {
            let file = std::fs::File::create(&archive_path).unwrap();
            let mut archive = zip::ZipWriter::new(file);
            let options = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Stored);
            archive
                .start_file(format!("jdk-21/bin/{binary_name}"), options)
                .unwrap();
            archive.write_all(b"test java binary").unwrap();
            archive.finish().unwrap();
        }

        let os = if cfg!(windows) {
            Os::Windows
        } else {
            Os::Linux
        };
        let destination = dir.path().join("extracted");
        let home = extract_archive(&archive_path, &destination, os).unwrap();

        assert_eq!(home, destination.join("jdk-21"));
        assert!(home.join("bin").join(binary_name).is_file());
    }

    #[test]
    fn extract_archive_rejects_data_beyond_declared_size() {
        let dir = tempfile::tempdir().unwrap();
        let archive_path = dir.path().join("jdk.zip");

        {
            let file = std::fs::File::create(&archive_path).unwrap();
            let mut archive = zip::ZipWriter::new(file);
            archive
                .start_file(
                    "jdk-21/bin/java",
                    zip::write::SimpleFileOptions::default()
                        .compression_method(zip::CompressionMethod::Stored),
                )
                .unwrap();
            archive.write_all(b"binary").unwrap();
            archive.finish().unwrap();
        }

        let mut bytes = std::fs::read(&archive_path).unwrap();
        let central_header = bytes
            .windows(4)
            .position(|window| window == b"PK\x01\x02")
            .unwrap();
        bytes[central_header + 24..central_header + 28].copy_from_slice(&1_u32.to_le_bytes());
        std::fs::write(&archive_path, bytes).unwrap();

        let destination = dir.path().join("extracted");
        let error = extract_archive(&archive_path, &destination, Os::Linux).unwrap_err();

        assert!(error.to_string().contains("declared size"));
    }

    #[test]
    fn extract_archive_rejects_path_traversal() {
        let dir = tempfile::tempdir().unwrap();
        let archive_path = dir.path().join("jdk.zip");

        {
            let file = std::fs::File::create(&archive_path).unwrap();
            let mut archive = zip::ZipWriter::new(file);
            archive
                .start_file("../escape", zip::write::SimpleFileOptions::default())
                .unwrap();
            archive.write_all(b"escaped").unwrap();
            archive.finish().unwrap();
        }

        let destination = dir.path().join("extracted");
        let error = extract_archive(&archive_path, &destination, Os::Linux).unwrap_err();

        assert!(error.to_string().contains("unsafe path"));
        assert!(!dir.path().join("escape").exists());
    }

    #[test]
    fn extract_archive_rejects_symlinks() {
        let dir = tempfile::tempdir().unwrap();
        let archive_path = dir.path().join("jdk.zip");

        {
            let file = std::fs::File::create(&archive_path).unwrap();
            let mut archive = zip::ZipWriter::new(file);
            archive
                .add_symlink(
                    "jdk-21/bin/java",
                    "../../../escape",
                    zip::write::SimpleFileOptions::default(),
                )
                .unwrap();
            archive.finish().unwrap();
        }

        let destination = dir.path().join("extracted");
        let error = extract_archive(&archive_path, &destination, Os::Linux).unwrap_err();

        assert!(error.to_string().contains("symbolic links"));
        assert!(!dir.path().join("escape").exists());
    }

    #[test]
    fn extract_archive_rejects_non_canonical_paths() {
        let dir = tempfile::tempdir().unwrap();
        let archive_path = dir.path().join("jdk.zip");

        {
            let file = std::fs::File::create(&archive_path).unwrap();
            let mut archive = zip::ZipWriter::new(file);
            let options = zip::write::SimpleFileOptions::default();
            archive
                .start_file("jdk-21/bin/../bin/java", options)
                .unwrap();
            archive.write_all(b"java binary").unwrap();
            archive.finish().unwrap();
        }

        let destination = dir.path().join("extracted");
        let error = extract_archive(&archive_path, &destination, Os::Linux).unwrap_err();

        assert!(error.to_string().contains("non-canonical path"));
    }

    #[test]
    fn extract_archive_rejects_case_insensitive_path_collisions() {
        let dir = tempfile::tempdir().unwrap();
        let archive_path = dir.path().join("jdk.zip");

        {
            let file = std::fs::File::create(&archive_path).unwrap();
            let mut archive = zip::ZipWriter::new(file);
            let options = zip::write::SimpleFileOptions::default();
            archive.start_file("jdk-21/bin/java", options).unwrap();
            archive.write_all(b"first").unwrap();
            archive.start_file("JDK-21/BIN/JAVA", options).unwrap();
            archive.write_all(b"second").unwrap();
            archive.finish().unwrap();
        }

        let destination = dir.path().join("extracted");
        let error = extract_archive(&archive_path, &destination, Os::Linux).unwrap_err();

        assert!(error.to_string().contains("duplicate ZIP entry path"));
    }

    #[test]
    fn extract_archive_rejects_windows_unsafe_components() {
        let dir = tempfile::tempdir().unwrap();
        let archive_path = dir.path().join("jdk.zip");

        {
            let file = std::fs::File::create(&archive_path).unwrap();
            let mut archive = zip::ZipWriter::new(file);
            archive
                .start_file(
                    "jdk-21/bin/java.exe:payload",
                    zip::write::SimpleFileOptions::default(),
                )
                .unwrap();
            archive.write_all(b"payload").unwrap();
            archive.finish().unwrap();
        }

        let destination = dir.path().join("extracted");
        let error = extract_archive(&archive_path, &destination, Os::Linux).unwrap_err();

        assert!(error.to_string().contains("non-portable path"));
    }

    #[test]
    fn zip_validation_limits_entry_count_before_allocating() {
        let dir = tempfile::tempdir().unwrap();
        let archive_path = dir.path().join("jdk.zip");

        {
            let file = std::fs::File::create(&archive_path).unwrap();
            let mut archive = zip::ZipWriter::new(file);
            archive
                .start_file("jdk/bin/java", zip::write::SimpleFileOptions::default())
                .unwrap();
            archive.write_all(b"binary").unwrap();
            archive.finish().unwrap();
        }

        let file = std::fs::File::open(&archive_path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        let error = validate_zip_archive_with_limits(&mut archive, 0, 1024, 1024).unwrap_err();

        assert!(error.to_string().contains("too many entries"));
    }

    #[test]
    fn zip_validation_limits_uncompressed_entry_size() {
        let dir = tempfile::tempdir().unwrap();
        let archive_path = dir.path().join("jdk.zip");

        {
            let file = std::fs::File::create(&archive_path).unwrap();
            let mut archive = zip::ZipWriter::new(file);
            archive
                .start_file("jdk/bin/java", zip::write::SimpleFileOptions::default())
                .unwrap();
            archive.write_all(b"binary").unwrap();
            archive.finish().unwrap();
        }

        let file = std::fs::File::open(&archive_path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        let error = validate_zip_archive_with_limits(&mut archive, 10, 5, 1024).unwrap_err();

        assert!(error.to_string().contains("entry is too large"));
    }

    #[test]
    fn zip_validation_limits_total_uncompressed_size() {
        let dir = tempfile::tempdir().unwrap();
        let archive_path = dir.path().join("jdk.zip");

        {
            let file = std::fs::File::create(&archive_path).unwrap();
            let mut archive = zip::ZipWriter::new(file);
            let options = zip::write::SimpleFileOptions::default();
            archive.start_file("jdk/bin/java", options).unwrap();
            archive.write_all(b"four").unwrap();
            archive.start_file("jdk/bin/javac", options).unwrap();
            archive.write_all(b"four").unwrap();
            archive.finish().unwrap();
        }

        let file = std::fs::File::open(&archive_path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        let error = validate_zip_archive_with_limits(&mut archive, 10, 4, 7).unwrap_err();

        assert!(error.to_string().contains("expands beyond"));
    }

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
