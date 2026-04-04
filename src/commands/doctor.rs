use anyhow::Result;
use console::style;
use jm_core::config::Config;
use jm_core::platform::Platform;
use jm_core::registry::Registry;
use jm_core::storage::StorageDirs;
use jm_install::link;

use crate::output;

struct Check {
    name: &'static str,
    passed: bool,
    detail: String,
}

pub async fn run() -> Result<()> {
    let mut checks: Vec<Check> = Vec::new();

    // 1. Platform detection
    checks.push(check_platform());

    // 2. Storage directories
    let dirs_result = StorageDirs::resolve();
    checks.push(check_storage_dirs(&dirs_result));

    if let Ok(ref dirs) = dirs_result {
        // 3. Config file
        checks.push(check_config(dirs));

        // 4. Registry
        checks.push(check_registry(dirs));

        // 5. Current symlink
        checks.push(check_current_link(dirs));

        // 6. Installed JDKs integrity
        checks.push(check_installed_jdks(dirs));

        // 7. API connectivity
        checks.push(check_api_connectivity(dirs).await);

        // 8. Shell integration
        checks.push(check_shell_integration(dirs));
    }

    // Print results
    println!(
        "\n{}",
        style("jm doctor — diagnostic report").bold().underlined()
    );
    println!();

    let mut pass_count = 0;
    let mut fail_count = 0;

    for check in &checks {
        let icon = if check.passed {
            pass_count += 1;
            style("PASS").green().bold()
        } else {
            fail_count += 1;
            style("FAIL").red().bold()
        };
        println!("  [{}] {}", icon, check.name);
        if !check.detail.is_empty() {
            for line in check.detail.lines() {
                println!("         {}", style(line).dim());
            }
        }
    }

    println!();
    if fail_count == 0 {
        output::print_success(&format!("All {} checks passed", pass_count));
    } else {
        output::print_warning(&format!("{} passed, {} failed", pass_count, fail_count));
    }

    Ok(())
}

fn check_platform() -> Check {
    match Platform::current() {
        Ok(p) => Check {
            name: "Platform detection",
            passed: true,
            detail: format!("{}", p),
        },
        Err(e) => Check {
            name: "Platform detection",
            passed: false,
            detail: format!("{}", e),
        },
    }
}

fn check_storage_dirs(dirs_result: &Result<StorageDirs, jm_core::error::JmError>) -> Check {
    match dirs_result {
        Ok(dirs) => {
            let exists = dirs.data_dir.exists();
            Check {
                name: "Storage directories",
                passed: exists,
                detail: if exists {
                    format!("data: {}", dirs.data_dir.display())
                } else {
                    format!(
                        "{} does not exist. Run 'jm install <version>' to create it.",
                        dirs.data_dir.display()
                    )
                },
            }
        }
        Err(e) => Check {
            name: "Storage directories",
            passed: false,
            detail: format!("{}", e),
        },
    }
}

fn check_config(dirs: &StorageDirs) -> Check {
    let path = dirs.config_path();
    if !path.exists() {
        return Check {
            name: "Configuration file",
            passed: true,
            detail: "Using defaults (no config.toml)".to_string(),
        };
    }
    match Config::load(dirs) {
        Ok(_) => Check {
            name: "Configuration file",
            passed: true,
            detail: format!("{}", path.display()),
        },
        Err(e) => Check {
            name: "Configuration file",
            passed: false,
            detail: format!("Parse error: {}", e),
        },
    }
}

fn check_registry(dirs: &StorageDirs) -> Check {
    match Registry::load(dirs) {
        Ok(reg) => Check {
            name: "JDK registry",
            passed: true,
            detail: format!("{} JDK(s) registered", reg.installations.len()),
        },
        Err(e) => Check {
            name: "JDK registry",
            passed: false,
            detail: format!("Failed to load: {}", e),
        },
    }
}

fn check_current_link(dirs: &StorageDirs) -> Check {
    let link_path = dirs.current_link();
    match link::read_current_link(&link_path) {
        Ok(Some(target)) => {
            let valid = target.exists();
            Check {
                name: "Global default (current link)",
                passed: valid,
                detail: if valid {
                    format!(
                        "-> {}",
                        target
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| target.display().to_string())
                    )
                } else {
                    format!(
                        "Broken link -> {}. Run 'jm default <version>' to fix.",
                        target.display()
                    )
                },
            }
        }
        Ok(None) => Check {
            name: "Global default (current link)",
            passed: true,
            detail: "Not set (use 'jm default <version>')".to_string(),
        },
        Err(e) => Check {
            name: "Global default (current link)",
            passed: false,
            detail: format!("{}", e),
        },
    }
}

fn check_installed_jdks(dirs: &StorageDirs) -> Check {
    let registry = match Registry::load(dirs) {
        Ok(r) => r,
        Err(_) => {
            return Check {
                name: "Installed JDKs integrity",
                passed: true,
                detail: "No registry".to_string(),
            }
        }
    };

    let mut missing = Vec::new();
    for inst in &registry.installations {
        let java_bin = if cfg!(windows) {
            inst.path.join("bin").join("java.exe")
        } else {
            inst.path.join("bin").join("java")
        };
        if !java_bin.exists() {
            missing.push(inst.id.clone());
        }
    }

    if missing.is_empty() {
        Check {
            name: "Installed JDKs integrity",
            passed: true,
            detail: format!(
                "All {} installation(s) have valid bin/java",
                registry.installations.len()
            ),
        }
    } else {
        Check {
            name: "Installed JDKs integrity",
            passed: false,
            detail: format!(
                "Missing bin/java in: {}. Reinstall with 'jm install'.",
                missing.join(", ")
            ),
        }
    }
}

async fn check_api_connectivity(dirs: &StorageDirs) -> Check {
    let config = Config::load(dirs).unwrap_or_default();
    let mut builder = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5));
    if let Some(ref proxy_url) = config.api.proxy {
        if let Ok(proxy) = reqwest::Proxy::all(proxy_url) {
            builder = builder.proxy(proxy);
        }
    }
    let client = builder.build();

    let client = match client {
        Ok(c) => c,
        Err(e) => {
            return Check {
                name: "API connectivity (Foojay Disco)",
                passed: false,
                detail: format!("Failed to create HTTP client: {}", e),
            }
        }
    };

    let url = format!(
        "{}/major_versions?maintained=true",
        config.api.disco_api_url
    );
    match client.get(&url).send().await {
        Ok(resp) if resp.status().is_success() => Check {
            name: "API connectivity (Foojay Disco)",
            passed: true,
            detail: config.api.disco_api_url,
        },
        Ok(resp) => Check {
            name: "API connectivity (Foojay Disco)",
            passed: false,
            detail: format!("HTTP {}", resp.status()),
        },
        Err(e) => Check {
            name: "API connectivity (Foojay Disco)",
            passed: false,
            detail: format!("{}", e),
        },
    }
}

fn check_shell_integration(dirs: &StorageDirs) -> Check {
    let java_home = std::env::var("JAVA_HOME").ok();
    let path = std::env::var("PATH").unwrap_or_default();
    let data_dir_str = dirs.data_dir.display().to_string();

    let shell_active = path.contains(&data_dir_str)
        || java_home
            .as_deref()
            .is_some_and(|h| h.contains(&data_dir_str));

    if shell_active {
        Check {
            name: "Shell integration",
            passed: true,
            detail: format!(
                "JAVA_HOME={}",
                java_home.unwrap_or_else(|| "(not set)".into())
            ),
        }
    } else {
        Check {
            name: "Shell integration",
            passed: false,
            detail: "jm not detected in PATH/JAVA_HOME. Add 'eval \"$(jm shell init <shell>)\"' to your shell rc file.".to_string(),
        }
    }
}
