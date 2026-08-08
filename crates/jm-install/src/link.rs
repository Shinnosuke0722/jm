use jm_core::error::Result;
use std::path::Path;

/// Update the `current` symlink to point to the specified JDK directory.
/// Uses atomic replacement: create a temp symlink then rename over the target.
#[cfg(unix)]
pub fn set_current_link(link_path: &Path, target: &Path) -> Result<()> {
    let parent = link_path.parent().unwrap_or_else(|| Path::new("."));

    // Create a temporary symlink with a unique name in the same directory
    let tmp_link = parent.join(format!(".current.tmp.{}", std::process::id()));

    // Clean up any stale temp link from a previous crash
    let _ = std::fs::remove_file(&tmp_link);

    std::os::unix::fs::symlink(target, &tmp_link)?;

    // Atomic rename: replaces link_path in one operation
    std::fs::rename(&tmp_link, link_path)?;

    Ok(())
}

/// Update the `current` junction to point to the specified JDK directory.
#[cfg(windows)]
pub fn set_current_link(link_path: &Path, target: &Path) -> Result<()> {
    let parent = link_path.parent().unwrap_or_else(|| Path::new("."));
    let tmp_link = parent.join(format!(".current.tmp.{}", std::process::id()));

    // A directory junction works without Developer Mode or administrator
    // privileges. Build it beside `current` first to keep the replacement
    // window as short as possible.
    remove_current_link(&tmp_link)?;
    if let Err(error) = junction::create(target, &tmp_link) {
        let _ = std::fs::remove_dir(&tmp_link);
        return Err(error.into());
    }

    if let Err(error) = remove_current_link(link_path) {
        let _ = remove_current_link(&tmp_link);
        return Err(error);
    }
    if let Err(error) = std::fs::rename(&tmp_link, link_path) {
        let _ = remove_current_link(&tmp_link);
        return Err(error.into());
    }

    Ok(())
}

/// Remove the `current` symlink or junction without touching its target.
#[cfg(unix)]
pub fn remove_current_link(link_path: &Path) -> Result<()> {
    match link_path.symlink_metadata() {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            std::fs::remove_file(link_path)?;
            Ok(())
        }
        Ok(_) => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "refusing to remove non-symlink path: {}",
                link_path.display()
            ),
        )
        .into()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

/// Remove the `current` symlink or junction without touching its target.
#[cfg(windows)]
pub fn remove_current_link(link_path: &Path) -> Result<()> {
    let metadata = match link_path.symlink_metadata() {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.into()),
    };

    // `junction::exists` follows the target and therefore returns false for a
    // broken junction. `get_target` inspects the reparse point directly.
    if junction::get_target(link_path).is_ok() {
        junction::delete(link_path)?;
        std::fs::remove_dir(link_path)?;
        return Ok(());
    }

    if metadata.file_type().is_symlink() {
        // Directory symlinks require RemoveDirectoryW, while file symlinks use
        // DeleteFileW. Trying both also handles a broken directory symlink.
        if std::fs::remove_dir(link_path).is_err() {
            std::fs::remove_file(link_path)?;
        }
        return Ok(());
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        format!("refusing to remove non-link path: {}", link_path.display()),
    )
    .into())
}

/// Read the target of the `current` symlink or junction.
#[cfg(unix)]
pub fn read_current_link(link_path: &Path) -> Result<Option<std::path::PathBuf>> {
    if !link_path.exists() && link_path.symlink_metadata().is_err() {
        return Ok(None);
    }
    let target = std::fs::read_link(link_path)?;
    Ok(Some(target))
}

/// Read the target of the `current` symlink or junction.
#[cfg(windows)]
pub fn read_current_link(link_path: &Path) -> Result<Option<std::path::PathBuf>> {
    match link_path.symlink_metadata() {
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    }

    if let Ok(target) = junction::get_target(link_path) {
        return Ok(Some(target));
    }

    Ok(Some(std::fs::read_link(link_path)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_and_read_link() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("jdk-21");
        std::fs::create_dir(&target).unwrap();
        let link = dir.path().join("current");

        set_current_link(&link, &target).unwrap();
        let read = read_current_link(&link).unwrap().unwrap();
        assert_eq!(read, target);
    }

    #[test]
    fn update_existing_link() {
        let dir = tempfile::tempdir().unwrap();
        let target1 = dir.path().join("jdk-17");
        let target2 = dir.path().join("jdk-21");
        std::fs::create_dir(&target1).unwrap();
        std::fs::create_dir(&target2).unwrap();
        let link = dir.path().join("current");

        set_current_link(&link, &target1).unwrap();
        set_current_link(&link, &target2).unwrap();
        let read = read_current_link(&link).unwrap().unwrap();
        assert_eq!(read, target2);
    }

    #[test]
    fn atomic_replacement_no_stale_temp() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("jdk-21");
        std::fs::create_dir(&target).unwrap();
        let link = dir.path().join("current");

        set_current_link(&link, &target).unwrap();

        // No stale .current.tmp.* should remain
        let temps: Vec<_> = std::fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().starts_with(".current.tmp."))
            .collect();
        assert!(temps.is_empty());
    }

    #[test]
    fn remove_link_keeps_target() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("jdk-21");
        std::fs::create_dir(&target).unwrap();
        let link = dir.path().join("current");

        set_current_link(&link, &target).unwrap();
        remove_current_link(&link).unwrap();

        assert!(target.exists());
        assert!(link.symlink_metadata().is_err());
    }

    #[test]
    fn read_and_remove_broken_link() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("jdk-21");
        std::fs::create_dir(&target).unwrap();
        let link = dir.path().join("current");

        set_current_link(&link, &target).unwrap();
        std::fs::remove_dir(&target).unwrap();

        let read = read_current_link(&link).unwrap().unwrap();
        assert_eq!(read, target);
        remove_current_link(&link).unwrap();
        assert!(link.symlink_metadata().is_err());
    }
}
