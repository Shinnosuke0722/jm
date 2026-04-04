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

    let _ = std::fs::remove_dir(&tmp_link);
    std::os::windows::fs::symlink_dir(target, &tmp_link)?;

    // On Windows, rename over an existing symlink_dir may fail,
    // so remove the old one first, then rename. Not perfectly atomic
    // but much smaller race window.
    if link_path.exists() || link_path.symlink_metadata().is_ok() {
        let _ = std::fs::remove_dir(link_path);
    }
    std::fs::rename(&tmp_link, link_path)?;

    Ok(())
}

/// Read the target of the `current` symlink.
pub fn read_current_link(link_path: &Path) -> Result<Option<std::path::PathBuf>> {
    if !link_path.exists() && link_path.symlink_metadata().is_err() {
        return Ok(None);
    }
    let target = std::fs::read_link(link_path)?;
    Ok(Some(target))
}

#[cfg(test)]
#[cfg(unix)]
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
}
