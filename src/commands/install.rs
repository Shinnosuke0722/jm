use anyhow::{Context, Result};
use console::style;
use jm_api::adoptium::AdoptiumClient;
use jm_api::cache::ApiCache;
use jm_api::disco::DiscoClient;
use jm_api::models::JdkPackage;
use jm_api::provider::{JdkProvider, PackageQuery};
use jm_core::config::Config;
use jm_core::distribution::Distribution;
use jm_core::java_version::{JavaVersion, VersionSpec};
use jm_core::platform::Platform;
use jm_core::registry::{Installation, Registry};
use jm_core::storage::StorageDirs;
use jm_install::{download, extract, link, verify};

use crate::output;

pub async fn run(
    version: &str,
    distribution_override: Option<&str>,
    set_default: bool,
    set_local: bool,
    no_verify: bool,
) -> Result<()> {
    let dirs = StorageDirs::resolve()?;
    dirs.ensure_dirs()?;
    let config = Config::load(&dirs)?;
    let platform = Platform::current()?;

    // Parse version spec
    let mut spec = VersionSpec::parse(version)?;
    if spec.distribution.is_none() {
        if let Some(dist) = distribution_override {
            spec.distribution = Some(Distribution::parse(dist));
        } else {
            spec.distribution = Some(Distribution::parse(&config.global.preferred_distribution));
        }
    }
    let dist = spec.distribution.as_ref().unwrap();

    output::print_info(&format!(
        "Searching for {} JDK {}...",
        style(dist.display_name()).cyan(),
        style(&spec.version).cyan()
    ));

    // Build query
    let query = PackageQuery {
        major_version: Some(spec.version.major),
        distribution: Some(dist.api_parameter().to_string()),
        operating_system: Some(platform.os.api_parameter().to_string()),
        architecture: Some(platform.arch.api_parameter().to_string()),
        archive_type: Some(platform.os.default_archive_type().to_string()),
        package_type: Some("jdk".to_string()),
        release_status: Some("ga".to_string()),
        latest: Some("per_distribution".to_string()),
    };

    // Query Disco API (primary), fallback to Adoptium on failure
    let cache = ApiCache::new(dirs.api_cache_dir());
    let disco = DiscoClient::new(config.api.disco_api_url.clone(), cache)?;

    let (provider_name, package, provider): (&str, JdkPackage, Box<dyn JdkProvider>) =
        match disco.query_packages(&query).await {
            Ok(packages) if !packages.is_empty() => {
                let pkg = packages.into_iter().next().unwrap();
                ("Foojay Disco", pkg, Box::new(disco))
            }
            Ok(_) | Err(_) if config.api.fallback_enabled && dist == &Distribution::Temurin => {
                output::print_warning("Foojay Disco unavailable, falling back to Adoptium API...");
                let adoptium = AdoptiumClient::new()?;
                let packages = adoptium.query_packages(&query).await?;
                let pkg = packages
                    .into_iter()
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("No matching JDK found via Adoptium either"))?;
                ("Adoptium", pkg, Box::new(adoptium))
            }
            Ok(_) => {
                anyhow::bail!("No matching JDK package found for {}", spec);
            }
            Err(e) => {
                return Err(e.into());
            }
        };

    output::print_info(&format!(
        "Found {} ({}) via {}",
        style(&package.java_version).green(),
        &package.filename,
        provider_name,
    ));

    // Check if already installed
    let mut registry = Registry::load(&dirs)?;
    let install_id = format!("{}-{}", dist.api_parameter(), &package.java_version);
    if registry.find_by_id(&install_id).is_some() {
        output::print_warning(&format!("{} is already installed", install_id));
        return Ok(());
    }

    // Resolve download URL
    let download_info = provider.resolve_download(&package).await?;

    // Download
    output::print_info(&format!("Downloading {}...", &download_info.filename));
    let archive_path = download::download_jdk(&download_info, &dirs.downloads_dir()).await?;

    // Verify checksum
    if !no_verify {
        if let Some(ref checksum) = download_info.checksum_sha256 {
            output::print_info("Verifying checksum...");
            verify::verify_sha256(&archive_path, checksum)?;
            output::print_success("Checksum verified");
        } else {
            output::print_warning("No checksum available, skipping verification");
        }
    }

    // Extract
    output::print_info("Extracting...");
    let temp_dir = tempfile::tempdir()?;
    let jdk_home = extract::extract_archive(&archive_path, temp_dir.path(), platform.os)?;

    // Move to final location
    let final_dir = dirs.jdks_dir().join(&install_id);
    if final_dir.exists() {
        std::fs::remove_dir_all(&final_dir)?;
    }
    move_dir(&jdk_home, &final_dir)?;

    // Register installation
    let installation = Installation {
        id: install_id.clone(),
        distribution: dist.clone(),
        java_version: JavaVersion::parse(&package.java_version)
            .unwrap_or_else(|_| JavaVersion::new(package.major_version)),
        full_version: package.java_version.clone(),
        major_version: package.major_version,
        path: final_dir.clone(),
        installed_at: chrono::Utc::now(),
        is_lts: package.term_of_support.to_uppercase() == "LTS",
    };
    registry.add(installation);
    registry.save(&dirs)?;

    // Clean up downloaded archive if configured
    if !config.install.keep_archives {
        let _ = std::fs::remove_file(&archive_path);
    }

    // Set as default if requested or if it's the first installation
    if set_default || registry.installations.len() == 1 {
        link::set_current_link(&dirs.current_link(), &final_dir)?;
        output::print_success(&format!("Set {} as global default", &install_id));
    }

    // Write .java-version if requested
    if set_local {
        let java_version_content = format!("{}\n", &install_id);
        std::fs::write(".java-version", java_version_content)?;
        output::print_success("Created .java-version file");
    }

    output::print_success(&format!(
        "Installed {} at {}",
        style(&install_id).green().bold(),
        final_dir.display()
    ));

    Ok(())
}

fn move_dir(src: &std::path::Path, dest: &std::path::Path) -> Result<()> {
    // Try rename first (fast, same filesystem)
    if std::fs::rename(src, dest).is_ok() {
        return Ok(());
    }
    // Fallback: recursive copy + remove
    copy_dir_recursive(src, dest).context("failed to copy JDK directory")?;
    let _ = std::fs::remove_dir_all(src);
    Ok(())
}

fn copy_dir_recursive(src: &std::path::Path, dest: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dest)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dest_path)?;
        } else {
            std::fs::copy(&src_path, &dest_path)?;
        }
    }
    Ok(())
}
