use anyhow::Result;
use jm_core::storage::StorageDirs;
use jm_shell::hook::Shell;

use crate::output;

pub fn init(shell_name: &str) -> Result<()> {
    let shell = Shell::parse(shell_name).ok_or_else(|| {
        anyhow::anyhow!(
            "Unknown shell: '{}'. Supported: bash, zsh, fish, powershell",
            shell_name
        )
    })?;

    let dirs = StorageDirs::resolve()?;
    let data_dir = dirs.data_dir.display().to_string();
    print!("{}", shell.init_script(&data_dir));

    Ok(())
}

pub fn completions(shell_name: &str) -> Result<()> {
    use clap::CommandFactory;
    use clap_complete::{Shell as ClapShell, generate};

    let shell = match shell_name.to_lowercase().as_str() {
        "bash" => ClapShell::Bash,
        "zsh" => ClapShell::Zsh,
        "fish" => ClapShell::Fish,
        "powershell" | "pwsh" => ClapShell::PowerShell,
        _ => {
            output::print_error(&format!(
                "Unknown shell: '{}'. Supported: bash, zsh, fish, powershell",
                shell_name
            ));
            return Ok(());
        }
    };

    let mut cmd = crate::cli::Cli::command();
    generate(shell, &mut cmd, "jm", &mut std::io::stdout());

    Ok(())
}
