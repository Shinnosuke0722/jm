use jm_core::error::Result;
use std::path::Path;

/// Update the `current` symlink to point to the specified JDK directory.
#[cfg(unix)]
pub fn set_current_link(link_path: &Path, target: &Path) -> Result<()> {
    // Remove existing symlink if present
    if link_path.exists() || link_path.symlink_metadata().is_ok() {
        std::fs::remove_file(link_path)?;
    }
    std::os::unix::fs::symlink(target, link_path)?;
    Ok(())
}

/// Update the `current` junction to point to the specified JDK directory.
#[cfg(windows)]
pub fn set_current_link(link_path: &Path, target: &Path) -> Result<()> {
    // Remove existing junction/directory if present
    if link_path.exists() {
        // Try to remove as junction first, then as directory
        let _ = std::fs::remove_dir(link_path);
        if link_path.exists() {
            std::fs::remove_dir_all(link_path)?;
        }
    }
    std::os::windows::fs::symlink_dir(target, link_path)?;
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
}
