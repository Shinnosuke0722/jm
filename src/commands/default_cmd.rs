use anyhow::Result;
use console::style;
use jm_core::java_version::VersionSpec;
use jm_core::registry::Registry;
use jm_core::storage::StorageDirs;
use jm_install::link;

use crate::output;

pub fn run(version: Option<&str>) -> Result<()> {
    let dirs = StorageDirs::resolve()?;

    match version {
        Some(version) => set_default(&dirs, version),
        None => show_default(&dirs),
    }
}

fn set_default(dirs: &StorageDirs, version: &str) -> Result<()> {
    let registry = Registry::load(dirs)?;
    let spec = VersionSpec::parse(version)?;
    let matches = registry.find_matching(spec.distribution.as_ref(), &spec.version);

    let installation = match matches.len() {
        0 => {
            output::print_error(&format!(
                "No installed JDK matches '{}'. Run 'jm install {}' first.",
                version, version
            ));
            return Ok(());
        }
        1 => matches[0],
        _ => {
            let mut sorted = matches;
            sorted.sort_by(|a, b| b.java_version.cmp(&a.java_version));
            sorted[0]
        }
    };

    link::set_current_link(&dirs.current_link(), &installation.path)?;
    output::print_success(&format!(
        "Global default set to {}",
        style(&installation.id).green().bold()
    ));

    Ok(())
}

fn show_default(dirs: &StorageDirs) -> Result<()> {
    let current = link::read_current_link(&dirs.current_link())?;
    match current {
        Some(target) => {
            let name = target
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| target.display().to_string());
            println!("{}", name);
        }
        None => {
            output::print_info("No global default set. Use 'jm default <version>' to set one.");
        }
    }
    Ok(())
}
