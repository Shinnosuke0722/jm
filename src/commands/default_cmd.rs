use anyhow::{Result, bail};
use console::style;
use jm_core::java_version::VersionSpec;
use jm_core::registry::Registry;
use jm_core::storage::StorageDirs;
use jm_install::link;

use crate::output;

pub async fn run(version: Option<&str>, install_if_missing: bool) -> Result<()> {
    let dirs = StorageDirs::resolve()?;

    match version {
        Some(version) => set_default(&dirs, version, install_if_missing).await,
        None if install_if_missing => bail!("--install requires a version"),
        None => show_default(&dirs),
    }
}

async fn set_default(dirs: &StorageDirs, version: &str, install_if_missing: bool) -> Result<()> {
    let registry = Registry::load(dirs)?;
    let spec = VersionSpec::parse(version)?;
    let matches = registry.find_matching(spec.distribution.as_ref(), &spec.version);

    let installation = match matches.len() {
        0 => {
            if install_if_missing {
                return super::install::run(version, None, true, false, false).await;
            }
            output::print_error(&format!(
                "No installed JDK matches '{version}'. Run 'jm install {version}' first."
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
            let name = target.file_name().map_or_else(
                || target.display().to_string(),
                |n| n.to_string_lossy().to_string(),
            );
            println!("{name}");
        }
        None => {
            output::print_info("No global default set. Use 'jm default <version>' to set one.");
        }
    }
    Ok(())
}
