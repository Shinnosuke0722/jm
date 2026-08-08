use anyhow::Result;
use console::style;
use jm_core::java_version::VersionSpec;
use jm_core::registry::Registry;
use jm_core::storage::StorageDirs;
use jm_install::link;

use crate::output;

pub async fn run(version: &str, global: bool, install_if_missing: bool) -> Result<()> {
    let dirs = StorageDirs::resolve()?;
    let registry = Registry::load(&dirs)?;

    let spec = VersionSpec::parse(version)?;
    let matches = registry.find_matching(spec.distribution.as_ref(), &spec.version);

    let installation = match matches.len() {
        0 => {
            if install_if_missing {
                return super::install::run(version, None, global, !global, false).await;
            }
            output::print_error(&format!(
                "No installed JDK matches '{}'. Run 'jm install {}' first.",
                version, version
            ));
            return Ok(());
        }
        1 => matches[0],
        _ => {
            // Pick the latest match
            let mut sorted = matches;
            sorted.sort_by(|a, b| b.java_version.cmp(&a.java_version));
            sorted[0]
        }
    };

    if global {
        // Set global default
        link::set_current_link(&dirs.current_link(), &installation.path)?;
        output::print_success(&format!(
            "Set global default to {}",
            style(&installation.id).green().bold()
        ));
    } else {
        // Write .java-version file
        let content = format!("{}\n", installation.id);
        std::fs::write(".java-version", content)?;
        output::print_success(&format!(
            "Set {} for current project (wrote .java-version)",
            style(&installation.id).green().bold()
        ));
    }

    Ok(())
}
