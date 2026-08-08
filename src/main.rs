mod cli;
mod commands;
mod output;

use clap::Parser;
use cli::{Cli, Commands, ConfigCommands, ShellCommands};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result: anyhow::Result<()> = match cli.command {
        Commands::Install {
            version,
            distribution,
            set_default,
            set_local,
            no_verify,
        } => {
            commands::install::run(
                &version,
                distribution.as_deref(),
                set_default,
                set_local,
                no_verify,
            )
            .await
        }
        Commands::Uninstall { version, force } => commands::uninstall::run(&version, force),
        Commands::List {
            remote,
            distribution,
            major,
            lts,
        } => commands::list::run(remote, distribution.as_deref(), major, lts).await,
        Commands::Search {
            query,
            distribution,
            os,
            arch,
            limit,
        } => {
            commands::search::run(
                &query,
                distribution.as_deref(),
                os.as_deref(),
                arch.as_deref(),
                limit,
            )
            .await
        }
        Commands::Use {
            version,
            global,
            install,
        } => commands::use_cmd::run(&version, global, install).await,
        Commands::Default { version, install } => {
            commands::default_cmd::run(version.as_deref(), install).await
        }
        Commands::Current => commands::current::run(),
        Commands::Which { binary } => commands::which::run(&binary),
        Commands::Env {
            detect,
            shell,
            java_home_only,
        } => commands::env::run(detect, shell, java_home_only),
        Commands::Shell { command } => match command {
            ShellCommands::Init { shell } => commands::shell::init(&shell),
            ShellCommands::Completions { shell } => commands::shell::completions(&shell),
        },
        Commands::Config { command } => match command {
            ConfigCommands::List => commands::config::list(),
            ConfigCommands::Get { key } => commands::config::get(&key),
            ConfigCommands::Set { key, value } => commands::config::set(&key, &value),
            ConfigCommands::Path => commands::config::path(),
        },
        Commands::Doctor => commands::doctor::run().await,
        Commands::Upgrade { check } => commands::upgrade::run(check).await,
    };

    if let Err(e) = result {
        output::print_error(&format!("{:#}", e));
        std::process::exit(1);
    }
}
