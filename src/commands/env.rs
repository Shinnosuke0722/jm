use anyhow::Result;
use jm_core::java_version::VersionSpec;
use jm_core::project;
use jm_core::registry::{Installation, Registry};
use jm_core::storage::StorageDirs;
use jm_install::link;

pub fn run(detect: bool, shell: bool, java_home_only: bool) -> Result<()> {
    let dirs = StorageDirs::resolve()?;

    let java_home = if detect {
        resolve_java_home(&dirs)?
    } else {
        // Just use the global default
        link::read_current_link(&dirs.current_link())?
    };

    let java_home = match java_home {
        Some(p) => p,
        None => return Ok(()), // Silently exit if no JDK found (for shell hooks)
    };

    if java_home_only {
        println!("{}", java_home.display());
        return Ok(());
    }

    if shell {
        // Output eval-able shell statements
        println!("export JAVA_HOME=\"{}\"", java_home.display());
        println!("export PATH=\"{}/bin:$PATH\"", java_home.display());
    } else {
        println!("JAVA_HOME={}", java_home.display());
        println!("PATH={}/bin:$PATH", java_home.display());
    }

    Ok(())
}

fn resolve_java_home(dirs: &StorageDirs) -> Result<Option<std::path::PathBuf>> {
    let cwd = std::env::current_dir()?;

    // Check project-level detection
    if let Some(detected) = project::detect_version(&cwd)? {
        let registry = Registry::load(dirs)?;
        if let Some(inst) = newest_matching_installation(&registry, &detected.spec) {
            return Ok(Some(inst.path.clone()));
        }
    }

    // Fall back to global default
    Ok(link::read_current_link(&dirs.current_link())?)
}

fn newest_matching_installation<'a>(
    registry: &'a Registry,
    spec: &VersionSpec,
) -> Option<&'a Installation> {
    let mut matches = registry.find_matching(spec.distribution.as_ref(), &spec.version);
    matches.sort_by(|a, b| b.java_version.cmp(&a.java_version));
    matches.into_iter().next()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use jm_core::distribution::Distribution;
    use jm_core::java_version::JavaVersion;
    use std::path::PathBuf;

    fn installation(id: &str, version: &str) -> Installation {
        let java_version = JavaVersion::parse(version).unwrap();
        Installation {
            id: id.to_string(),
            distribution: Distribution::Temurin,
            java_version: java_version.clone(),
            full_version: version.to_string(),
            major_version: java_version.major,
            path: PathBuf::from(id),
            installed_at: Utc::now(),
            is_lts: true,
        }
    }

    #[test]
    fn project_detection_selects_the_newest_matching_installation() {
        let mut registry = Registry::empty();
        registry.add(installation("temurin-21.0.1", "21.0.1"));
        registry.add(installation("temurin-21.0.3", "21.0.3"));
        registry.add(installation("temurin-17.0.12", "17.0.12"));

        let spec = VersionSpec::parse("temurin-21").unwrap();
        let selected = newest_matching_installation(&registry, &spec).unwrap();

        assert_eq!(selected.id, "temurin-21.0.3");
    }
}
