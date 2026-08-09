use anyhow::{Result, bail};
use jm_core::config::Config;
use jm_core::storage::StorageDirs;

use crate::output;

pub fn list() -> Result<()> {
    let dirs = StorageDirs::resolve()?;
    let config = Config::load(&dirs)?;
    let toml_str = toml::to_string_pretty(&config)?;
    println!("{}", toml_str);
    Ok(())
}

pub fn get(key: &str) -> Result<()> {
    let dirs = StorageDirs::resolve()?;
    let config = Config::load(&dirs)?;

    let value = match key {
        "global.java_version" => config
            .global
            .java_version
            .unwrap_or_else(|| "(not set)".into()),
        "global.preferred_distribution" => config.global.preferred_distribution,
        "global.auto_install" => config.global.auto_install.to_string(),
        "install.verify_checksum" => config.install.verify_checksum.to_string(),
        "install.keep_archives" => config.install.keep_archives.to_string(),
        "api.disco_api_url" => config.api.disco_api_url,
        "api.fallback_enabled" => config.api.fallback_enabled.to_string(),
        "api.timeout" => config.api.timeout.to_string(),
        "api.cache_ttl_hours" => config.api.cache_ttl_hours.to_string(),
        "api.proxy" => config.api.proxy.unwrap_or_else(|| "(not set)".into()),
        "ui.color" => config.ui.color,
        "ui.progress" => config.ui.progress.to_string(),
        _ => bail!(
            "Unknown config key: '{}'. Run 'jm config list' to see all keys.",
            key
        ),
    };

    println!("{}", value);
    Ok(())
}

pub fn set(key: &str, value: &str) -> Result<()> {
    let dirs = StorageDirs::resolve()?;
    dirs.ensure_dirs()?;
    let mut config = Config::load(&dirs)?;

    match key {
        "global.java_version" => {
            config.global.java_version = if value == "null" || value.is_empty() {
                None
            } else {
                Some(value.to_string())
            };
        }
        "global.preferred_distribution" => {
            config.global.preferred_distribution = value.to_string();
        }
        "global.auto_install" => {
            config.global.auto_install = parse_bool(value)?;
        }
        "install.verify_checksum" => {
            config.install.verify_checksum = parse_bool(value)?;
        }
        "install.keep_archives" => {
            config.install.keep_archives = parse_bool(value)?;
        }
        "api.disco_api_url" => {
            config.api.disco_api_url = value.to_string();
        }
        "api.fallback_enabled" => {
            config.api.fallback_enabled = parse_bool(value)?;
        }
        "api.timeout" => {
            config.api.timeout = value
                .parse()
                .map_err(|_| anyhow::anyhow!("Expected a number"))?;
        }
        "api.cache_ttl_hours" => {
            config.api.cache_ttl_hours = value
                .parse()
                .map_err(|_| anyhow::anyhow!("Expected a number"))?;
        }
        "api.proxy" => {
            config.api.proxy = if value == "null" || value.is_empty() {
                None
            } else {
                Some(value.to_string())
            };
        }
        "ui.color" => {
            if !["auto", "always", "never"].contains(&value) {
                bail!("Invalid value for ui.color: must be auto, always, or never");
            }
            config.ui.color = value.to_string();
        }
        "ui.progress" => {
            config.ui.progress = parse_bool(value)?;
        }
        _ => bail!(
            "Unknown config key: '{}'. Run 'jm config list' to see all keys.",
            key
        ),
    }

    config.save(&dirs)?;
    output::print_success(&format!("Set {} = {}", key, value));
    Ok(())
}

pub fn path() -> Result<()> {
    let dirs = StorageDirs::resolve()?;
    println!("{}", dirs.config_path().display());
    Ok(())
}

fn parse_bool(s: &str) -> Result<bool> {
    match s {
        "true" | "1" | "yes" | "on" => Ok(true),
        "false" | "0" | "no" | "off" => Ok(false),
        _ => bail!("Expected boolean value (true/false)"),
    }
}
