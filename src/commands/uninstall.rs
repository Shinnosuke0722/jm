use anyhow::Result;
use console::style;
use jm_core::java_version::VersionSpec;
use jm_core::registry::Registry;
use jm_core::storage::StorageDirs;
use jm_install::link;

use crate::output;

pub fn run(version: &str, force: bool) -> Result<()> {
    let dirs = StorageDirs::resolve()?;
    let mut registry = Registry::load(&dirs)?;

    // Try exact id match first
    let install_id = if registry.find_by_id(version).is_some() {
        version.to_string()
    } else {
        // Try version spec matching
        let spec = VersionSpec::parse(version)?;
        let matches = registry.find_matching(spec.distribution.as_ref(), &spec.version);
        match matches.len() {
            0 => {
                output::print_error(&format!("No installed JDK matches '{}'", version));
                return Ok(());
            }
            1 => matches[0].id.clone(),
            _ => {
                output::print_error(&format!(
                    "Multiple JDKs match '{}'. Be more specific:",
                    version
                ));
                for m in matches {
                    eprintln!("  - {}", m.id);
                }
                return Ok(());
            }
        }
    };

    let installation = registry.find_by_id(&install_id).unwrap();

    if !force {
        output::print_warning(&format!(
            "About to uninstall {} from {}",
            style(&install_id).yellow(),
            installation.path.display()
        ));
        // In a real implementation, prompt for confirmation here
        // For now, proceed directly
    }

    let install_path = installation.path.clone();

    // Remove the JDK directory
    if install_path.exists() {
        std::fs::remove_dir_all(&install_path)?;
    }

    // Remove from registry
    registry.remove(&install_id);
    registry.save(&dirs)?;

    // If this was the current default, remove the link
    let current = link::read_current_link(&dirs.current_link())?;
    if let Some(current_target) = current {
        if current_target == install_path {
            let _ = std::fs::remove_file(dirs.current_link());
            output::print_warning("Removed global default (was pointing to uninstalled JDK)");
        }
    }

    output::print_success(&format!("Uninstalled {}", style(&install_id).green()));

    Ok(())
}
