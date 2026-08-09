use anyhow::Result;
use jm_api::cache::ApiCache;
use jm_api::disco::DiscoClient;
use jm_api::provider::{JdkProvider, PackageQuery};
use jm_core::config::Config;
use jm_core::distribution::Distribution;
use jm_core::platform::Platform;
use jm_core::registry::Registry;
use jm_core::storage::StorageDirs;
use jm_install::link;

use crate::output;

pub async fn run(
    remote: bool,
    distribution: Option<&str>,
    major: Option<u32>,
    lts: bool,
) -> Result<()> {
    let distribution = distribution
        .map(|value| {
            let parsed = Distribution::parse(value);
            parsed.validate()?;
            Ok::<_, jm_core::error::JmError>(parsed)
        })
        .transpose()?;

    if remote {
        list_remote(distribution.as_ref(), major, lts).await
    } else {
        list_local(distribution.as_ref(), major, lts)
    }
}

fn list_local(distribution: Option<&Distribution>, major: Option<u32>, lts: bool) -> Result<()> {
    let dirs = StorageDirs::resolve()?;
    let registry = Registry::load(&dirs)?;

    let current_target = link::read_current_link(&dirs.current_link())?.map(|p| {
        p.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default()
    });

    let mut installations = registry.sorted();

    // Apply filters
    if let Some(distribution) = distribution {
        installations.retain(|installation| &installation.distribution == distribution);
    }
    if let Some(major) = major {
        installations.retain(|i| i.major_version == major);
    }
    if lts {
        installations.retain(|i| i.is_lts);
    }

    if installations.is_empty() {
        output::print_info("No JDKs installed. Use 'jm install <version>' to install one.");
        return Ok(());
    }

    let mut table = output::create_table(&["Version", "Distribution", "Status", "Path"]);

    for inst in &installations {
        let is_current = current_target.as_deref() == Some(&inst.id);
        let version_display = output::format_version(&inst.full_version, is_current);
        let status = if is_current {
            output::format_lts(inst.is_lts) + " current"
        } else {
            output::format_lts(inst.is_lts)
        };

        table.add_row(vec![
            version_display,
            inst.distribution.display_name().to_string(),
            status,
            inst.path.display().to_string(),
        ]);
    }

    println!("{table}");
    Ok(())
}

async fn list_remote(
    distribution: Option<&Distribution>,
    major: Option<u32>,
    lts: bool,
) -> Result<()> {
    let dirs = StorageDirs::resolve()?;
    dirs.ensure_dirs()?;
    let config = Config::load(&dirs)?;
    let platform = Platform::current()?;

    let cache = ApiCache::new(dirs.api_cache_dir());
    let client =
        DiscoClient::with_proxy(config.api.disco_api_url, cache, config.api.proxy.as_deref())?;

    if let Some(major) = major {
        // List packages for a specific major version
        let query = PackageQuery {
            major_version: Some(major),
            version_requirement: None,
            distribution: distribution.map(|value| value.api_parameter().to_string()),
            operating_system: Some(platform.os.api_parameter().to_string()),
            architecture: Some(platform.arch.api_parameter().to_string()),
            archive_type: Some(platform.os.default_archive_type().to_string()),
            package_type: Some("jdk".to_string()),
            release_status: Some("ga".to_string()),
            latest: Some("per_distribution".to_string()),
        };

        let packages = client.query_packages(&query).await?;
        let mut table = output::create_table(&["Version", "Distribution", "Type", "Size"]);

        for pkg in &packages {
            let size = if pkg.size > 0 {
                format!("{:.0} MB", pkg.size as f64 / 1_048_576.0)
            } else {
                "-".to_string()
            };
            table.add_row(vec![
                pkg.java_version.clone(),
                pkg.distribution.clone(),
                pkg.term_of_support.clone(),
                size,
            ]);
        }
        println!("{table}");
    } else {
        // List major versions
        let versions = client.list_major_versions().await?;
        let mut table = output::create_table(&["Version", "Type", "Status"]);

        for v in &versions {
            if lts && v.term_of_support != "LTS" {
                continue;
            }
            if !v.maintained {
                continue;
            }
            let status = if v.early_access_only { "EA" } else { "GA" };
            table.add_row(vec![
                format!("JDK {}", v.major_version),
                v.term_of_support.clone(),
                status.to_string(),
            ]);
        }
        println!("{table}");
    }

    Ok(())
}
