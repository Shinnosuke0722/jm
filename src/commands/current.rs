use anyhow::Result;
use console::style;
use jm_core::project;
use jm_core::registry::Registry;
use jm_core::storage::StorageDirs;
use jm_install::link;

use crate::output;

pub fn run() -> Result<()> {
    let dirs = StorageDirs::resolve()?;

    // Check for project-level version detection
    let cwd = std::env::current_dir()?;
    if let Some(detected) = project::detect_version(&cwd)? {
        let source_desc = match &detected.source {
            project::VersionSource::EnvVar => "JM_JAVA_VERSION env var".to_string(),
            project::VersionSource::JavaVersionFile(p) => format!("{}", p.display()),
            project::VersionSource::SdkmanRc(p) => format!("{}", p.display()),
            project::VersionSource::GlobalDefault => "global default".to_string(),
        };

        println!(
            "{} (from {})",
            style(&detected.spec.to_string()).green().bold(),
            style(&source_desc).dim()
        );

        // Check if it's actually installed
        let registry = Registry::load(&dirs)?;
        let matches =
            registry.find_matching(detected.spec.distribution.as_ref(), &detected.spec.version);
        if matches.is_empty() {
            output::print_warning(&format!(
                "Version {} is not installed. Run 'jm install {}'",
                detected.spec, detected.spec
            ));
        }

        return Ok(());
    }

    // Fall back to global default
    let current = link::read_current_link(&dirs.current_link())?;
    match current {
        Some(target) => {
            let name = target
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| target.display().to_string());
            println!(
                "{} ({})",
                style(&name).green().bold(),
                style("global default").dim()
            );
        }
        None => {
            output::print_info("No JDK version active. Use 'jm install <version>' to get started.");
        }
    }

    Ok(())
}
