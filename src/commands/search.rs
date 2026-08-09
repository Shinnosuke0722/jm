use anyhow::Result;
use jm_api::cache::ApiCache;
use jm_api::disco::DiscoClient;
use jm_api::provider::{JdkProvider, PackageQuery};
use jm_core::config::Config;
use jm_core::distribution::Distribution;
use jm_core::platform::Platform;
use jm_core::storage::StorageDirs;

use crate::output;

pub async fn run(
    query: &str,
    distribution: Option<&str>,
    os_override: Option<&str>,
    arch_override: Option<&str>,
    limit: usize,
) -> Result<()> {
    let distribution = distribution
        .map(|value| {
            let parsed = Distribution::parse(value);
            parsed.validate()?;
            Ok::<_, jm_core::error::JmError>(parsed)
        })
        .transpose()?;

    // Determine if query is a version number or distribution name before any
    // network client is constructed, so invalid names fail locally.
    let (major_version, dist_filter) = if let Ok(version) = query.parse::<u32>() {
        (
            Some(version),
            distribution
                .as_ref()
                .map(|value| value.api_parameter().to_string()),
        )
    } else {
        let parsed = Distribution::parse(query);
        parsed.validate()?;
        (None, Some(parsed.api_parameter().to_string()))
    };

    let dirs = StorageDirs::resolve()?;
    dirs.ensure_dirs()?;
    let config = Config::load(&dirs)?;
    let platform = Platform::current()?;

    let cache = ApiCache::new(dirs.api_cache_dir());
    let client =
        DiscoClient::with_proxy(config.api.disco_api_url, cache, config.api.proxy.as_deref())?;

    let pkg_query = PackageQuery {
        major_version,
        version_requirement: None,
        distribution: dist_filter,
        operating_system: Some(
            os_override
                .unwrap_or(platform.os.api_parameter())
                .to_string(),
        ),
        architecture: Some(
            arch_override
                .unwrap_or(platform.arch.api_parameter())
                .to_string(),
        ),
        archive_type: Some(platform.os.default_archive_type().to_string()),
        package_type: Some("jdk".to_string()),
        release_status: Some("ga".to_string()),
        latest: Some("per_distribution".to_string()),
    };

    let packages = client.query_packages(&pkg_query).await?;

    if packages.is_empty() {
        output::print_info(&format!("No packages found matching '{}'", query));
        return Ok(());
    }

    let mut table = output::create_table(&["Version", "Distribution", "Type", "Arch", "Size"]);

    for pkg in packages.iter().take(limit) {
        let size = if pkg.size > 0 {
            format!("{:.0} MB", pkg.size as f64 / 1_048_576.0)
        } else {
            "-".to_string()
        };
        table.add_row(vec![
            pkg.java_version.clone(),
            pkg.distribution.clone(),
            pkg.term_of_support.clone(),
            pkg.architecture.clone(),
            size,
        ]);
    }

    println!("{table}");

    if packages.len() > limit {
        output::print_info(&format!(
            "Showing {} of {} results. Use --limit to see more.",
            limit,
            packages.len()
        ));
    }

    Ok(())
}
