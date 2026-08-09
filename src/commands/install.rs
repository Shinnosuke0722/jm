use anyhow::{bail, Context, Result};
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
use std::path::Path;

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

    // Parse the version spec and validate the selected distribution regardless
    // of whether it came from the spec, --distribution, or configuration.
    let mut spec = VersionSpec::parse(version)?;
    resolve_distribution(
        &mut spec,
        distribution_override,
        &config.global.preferred_distribution,
    )?;
    let dist = spec.distribution.as_ref().unwrap();
    let specific_version = has_specific_version(&spec.version);

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
        latest: (!specific_version).then(|| "per_distribution".to_string()),
    };

    // Query Disco API (primary), fallback to Adoptium on failure
    let cache = ApiCache::new(dirs.api_cache_dir());
    let proxy = config.api.proxy.as_deref();
    let disco = DiscoClient::with_proxy(config.api.disco_api_url.clone(), cache, proxy)?;

    let disco_result = disco.query_packages(&query).await;
    let disco_package = disco_result
        .as_ref()
        .ok()
        .and_then(|packages| find_matching_package(packages, &spec.version));

    let selection: (&str, JdkPackage, Box<dyn JdkProvider>);
    if let Some(package) = disco_package {
        selection = ("Foojay Disco", package, Box::new(disco));
    } else if config.api.fallback_enabled && dist == &Distribution::Temurin {
        output::print_warning(
            "Foojay Disco unavailable or returned no matching package; falling back to Adoptium API...",
        );
        let adoptium = AdoptiumClient::with_proxy(proxy)?;
        let packages = adoptium.query_packages(&query).await?;
        let package = find_matching_package(&packages, &spec.version)
            .ok_or_else(|| anyhow::anyhow!("No matching JDK found via Adoptium either"))?;
        selection = ("Adoptium", package, Box::new(adoptium));
    } else {
        match disco_result {
            Ok(_) => bail!("No matching JDK package found for {}", spec),
            Err(error) => return Err(error.into()),
        }
    }
    let (provider_name, package, provider) = selection;

    // Treat all provider-controlled path material as untrusted. Constructing
    // the destination through this helper guarantees it stays one lexical
    // child beneath the JDK installation directory.
    let (install_id, final_dir) =
        build_install_destination(&dirs.jdks_dir(), dist, &package.java_version)?;

    output::print_info(&format!(
        "Found {} ({}) via {}",
        style(&package.java_version).green(),
        package.filename,
        provider_name,
    ));

    // Check if already installed before the expensive download. A concurrent
    // `jm install` may have won the race after use/default first checked the
    // registry, so this path must still honor the requested selection flags.
    if let Some(existing) = Registry::load(&dirs)?.find_by_id(&install_id).cloned() {
        output::print_warning(&format!("{} is already installed", install_id));
        apply_requested_selection(
            &dirs,
            &existing.id,
            &existing.path,
            set_default,
            set_local,
            Path::new(".java-version"),
        )?;
        print_selection_success(&existing.id, set_default, set_local);
        return Ok(());
    }

    // Resolve download URL
    let download_info = provider.resolve_download(&package).await?;

    // Download
    output::print_info(&format!("Downloading {}...", download_info.filename));
    let archive_path =
        download::download_jdk_with_proxy(&download_info, &dirs.downloads_dir(), proxy).await?;

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
    if final_dir.exists() {
        std::fs::remove_dir_all(&final_dir)?;
    }
    move_dir(&jdk_home, &final_dir)?;

    // Register installation under exclusive lock (prevents concurrent corruption)
    let installation_count = Registry::locked_update(&dirs, |registry| {
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
        Ok(registry.installations.len())
    })?;

    // Clean up downloaded archive if configured
    if !config.install.keep_archives {
        let _ = std::fs::remove_file(&archive_path);
    }

    // Set as default if requested or if it's the first installation, and
    // write the project pin when requested.
    let make_default = set_default || installation_count == 1;
    apply_requested_selection(
        &dirs,
        &install_id,
        &final_dir,
        make_default,
        set_local,
        Path::new(".java-version"),
    )?;
    print_selection_success(&install_id, make_default, set_local);

    output::print_success(&format!(
        "Installed {} at {}",
        style(&install_id).green().bold(),
        final_dir.display()
    ));

    Ok(())
}

fn resolve_distribution(
    spec: &mut VersionSpec,
    distribution_override: Option<&str>,
    preferred_distribution: &str,
) -> Result<()> {
    if spec.distribution.is_none() {
        let selected = distribution_override.unwrap_or(preferred_distribution);
        spec.distribution = Some(Distribution::parse(selected));
    }

    spec.distribution
        .as_ref()
        .expect("distribution was assigned above")
        .validate()?;
    Ok(())
}

fn has_specific_version(version: &JavaVersion) -> bool {
    version.minor.is_some() || version.patch.is_some() || version.build.is_some()
}

fn find_matching_package(
    packages: &[JdkPackage],
    requested_version: &JavaVersion,
) -> Option<JdkPackage> {
    if !has_specific_version(requested_version) {
        return packages
            .iter()
            .find(|package| package.major_version == requested_version.major)
            .cloned();
    }

    packages
        .iter()
        .filter_map(|package| {
            let version = JavaVersion::parse(&package.java_version).ok()?;
            version
                .matches(requested_version)
                .then_some((package, version))
        })
        .max_by(|(_, left), (_, right)| left.cmp(right))
        .map(|(package, _)| package.clone())
}

fn build_install_destination(
    jdks_dir: &Path,
    distribution: &Distribution,
    java_version: &str,
) -> Result<(String, std::path::PathBuf)> {
    distribution.validate()?;
    if !download::is_safe_filename_component(java_version) {
        bail!(
            "unsafe Java version from provider {:?}: expected one ordinary filename component",
            java_version
        );
    }

    let install_id = format!("{}-{}", distribution.api_parameter(), java_version);
    if !download::is_safe_filename_component(&install_id) {
        bail!(
            "unsafe installation identifier {:?}: expected one ordinary filename component",
            install_id
        );
    }

    Ok((install_id.clone(), jdks_dir.join(install_id)))
}

fn apply_requested_selection(
    dirs: &StorageDirs,
    install_id: &str,
    install_path: &Path,
    set_default: bool,
    set_local: bool,
    local_version_path: &Path,
) -> Result<()> {
    if set_default {
        link::set_current_link(&dirs.current_link(), install_path)?;
    }
    if set_local {
        std::fs::write(local_version_path, format!("{}\n", install_id))?;
    }
    Ok(())
}

fn print_selection_success(install_id: &str, set_default: bool, set_local: bool) {
    if set_default {
        output::print_success(&format!("Set {} as global default", install_id));
    }
    if set_local {
        output::print_success("Created .java-version file");
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn package(java_version: &str, major_version: u32) -> JdkPackage {
        JdkPackage {
            id: format!("package-{java_version}"),
            distribution: "temurin".to_string(),
            major_version,
            java_version: java_version.to_string(),
            operating_system: "windows".to_string(),
            architecture: "x64".to_string(),
            archive_type: "zip".to_string(),
            filename: format!("jdk-{java_version}.zip"),
            term_of_support: "LTS".to_string(),
            directly_downloadable: true,
            size: 1,
        }
    }

    #[test]
    fn recognizes_version_requirements_that_need_full_package_results() {
        assert!(!has_specific_version(&JavaVersion::parse("21").unwrap()));
        assert!(has_specific_version(&JavaVersion::parse("21.0").unwrap()));
        assert!(has_specific_version(&JavaVersion::parse("21.0.2").unwrap()));
        assert!(has_specific_version(&JavaVersion::parse("21+13").unwrap()));
    }

    #[test]
    fn package_selection_preserves_minor_patch_and_build_requirements() {
        let packages = vec![
            package("21.0.12+8", 21),
            package("21.0.2+12", 21),
            package("21.0.2+13", 21),
            package("17.0.12+7", 17),
        ];

        let major = JavaVersion::parse("21").unwrap();
        assert_eq!(
            find_matching_package(&packages, &major)
                .unwrap()
                .java_version,
            "21.0.12+8"
        );

        let minor = JavaVersion::parse("21.0").unwrap();
        assert_eq!(
            find_matching_package(&packages, &minor)
                .unwrap()
                .java_version,
            "21.0.12+8"
        );

        let patch = JavaVersion::parse("21.0.2").unwrap();
        assert_eq!(
            find_matching_package(&packages, &patch)
                .unwrap()
                .java_version,
            "21.0.2+13"
        );

        let build = JavaVersion::parse("21.0.2+12").unwrap();
        assert_eq!(
            find_matching_package(&packages, &build)
                .unwrap()
                .java_version,
            "21.0.2+12"
        );

        for missing in ["21.0.3", "21.0.2+14"] {
            let missing = JavaVersion::parse(missing).unwrap();
            assert!(find_matching_package(&packages, &missing).is_none());
        }
    }

    #[test]
    fn validates_distribution_from_every_source() {
        let mut preferred = VersionSpec::parse("21").unwrap();
        assert!(resolve_distribution(&mut preferred, None, "../../escape").is_err());

        let mut overridden = VersionSpec::parse("21").unwrap();
        assert!(resolve_distribution(&mut overridden, Some("..\\escape"), "temurin").is_err());

        let mut explicit = VersionSpec::parse("custom_dist-21").unwrap();
        resolve_distribution(&mut explicit, None, "../../ignored").unwrap();
        assert_eq!(
            explicit.distribution.unwrap().api_parameter(),
            "custom_dist"
        );
    }

    #[test]
    fn builds_only_single_component_install_destinations() {
        let root = Path::new("jdks");
        let (id, path) =
            build_install_destination(root, &Distribution::Temurin, "21.0.8+9").unwrap();
        assert_eq!(id, "temurin-21.0.8+9");
        assert_eq!(path, root.join(&id));

        for version in [
            "",
            ".",
            "..",
            "../outside",
            "..\\outside",
            "/tmp/outside",
            "C:\\temp\\outside",
            "C:/temp/outside",
            "C:outside",
            "21:stream",
            "21\n",
            "21\0",
        ] {
            assert!(
                build_install_destination(root, &Distribution::Temurin, version).is_err(),
                "{version:?}"
            );
        }
    }

    #[test]
    fn already_installed_selection_honors_default_and_local_flags() {
        let temp = tempfile::tempdir().unwrap();
        let dirs = StorageDirs {
            data_dir: temp.path().join("data"),
            config_dir: temp.path().join("config"),
            cache_dir: temp.path().join("cache"),
        };
        dirs.ensure_dirs().unwrap();

        let install_id = "temurin-21.0.8+9";
        let install_path = dirs.jdks_dir().join(install_id);
        std::fs::create_dir(&install_path).unwrap();
        let local_version_path = temp.path().join("project.java-version");

        apply_requested_selection(
            &dirs,
            install_id,
            &install_path,
            true,
            true,
            &local_version_path,
        )
        .unwrap();

        assert_eq!(
            link::read_current_link(&dirs.current_link())
                .unwrap()
                .unwrap(),
            install_path
        );
        assert_eq!(
            std::fs::read_to_string(local_version_path).unwrap(),
            format!("{}\n", install_id)
        );
    }
}
