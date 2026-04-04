use anyhow::Result;
use jm_core::project;
use jm_core::registry::Registry;
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
        let matches =
            registry.find_matching(detected.spec.distribution.as_ref(), &detected.spec.version);
        if let Some(inst) = matches.first() {
            return Ok(Some(inst.path.clone()));
        }
    }

    // Fall back to global default
    Ok(link::read_current_link(&dirs.current_link())?)
}
