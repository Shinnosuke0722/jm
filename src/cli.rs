use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "jm",
    about = "A fast, cross-platform JDK version manager",
    version,
    after_help = "Examples:\n  jm install 21            Install latest JDK 21 (Temurin)\n  jm install temurin-21    Install Temurin JDK 21\n  jm list                  List installed JDKs\n  jm search 21             Search available JDK 21 packages\n  jm use 21                Set JDK 21 for current project\n  jm default 21            Set global default to JDK 21"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Suppress non-essential output
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Disable colored output
    #[arg(long, global = true)]
    pub no_color: bool,

    /// Output in JSON format
    #[arg(long, global = true)]
    pub json: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Install a JDK version
    #[command(visible_alias = "i")]
    Install {
        /// Version to install (e.g., 21, temurin-21, corretto-17.0.10)
        version: String,

        /// Override preferred distribution
        #[arg(short, long)]
        distribution: Option<String>,

        /// Set as global default after install
        #[arg(short = 's', long)]
        set_default: bool,

        /// Write .java-version file after install
        #[arg(short = 'l', long)]
        set_local: bool,

        /// Skip checksum verification
        #[arg(long)]
        no_verify: bool,
    },

    /// Remove an installed JDK version
    #[command(visible_alias = "rm")]
    Uninstall {
        /// Version to remove (e.g., temurin-21.0.2+13)
        version: String,

        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },

    /// List installed JDK versions
    #[command(visible_alias = "ls")]
    List {
        /// List available remote versions instead
        #[arg(short, long)]
        remote: bool,

        /// Filter by distribution
        #[arg(short, long)]
        distribution: Option<String>,

        /// Filter by major version
        #[arg(short, long)]
        major: Option<u32>,

        /// Show only LTS versions
        #[arg(long)]
        lts: bool,
    },

    /// Search available JDK versions across distributions
    Search {
        /// Search query (version number, distribution name, etc.)
        query: String,

        /// Filter by distribution
        #[arg(short, long)]
        distribution: Option<String>,

        /// Override detected OS
        #[arg(long)]
        os: Option<String>,

        /// Override detected architecture
        #[arg(long)]
        arch: Option<String>,

        /// Max results to display
        #[arg(long, default_value = "20")]
        limit: usize,
    },

    /// Set the JDK version for the current project
    #[command(name = "use")]
    Use {
        /// Version spec (e.g., 21, temurin-21)
        version: String,

        /// Set as global default instead
        #[arg(long)]
        global: bool,

        /// Install if not present, then set
        #[arg(long)]
        install: bool,
    },

    /// Set or display the global default JDK version
    Default {
        /// Version to set as default (omit to show current default)
        version: Option<String>,

        /// Install if not present, then set
        #[arg(long)]
        install: bool,
    },

    /// Show the currently active JDK version
    Current,

    /// Show the path to a JDK binary
    Which {
        /// Binary name (e.g., java, javac, jar)
        #[arg(default_value = "java")]
        binary: String,
    },

    /// Print JAVA_HOME and PATH for the active JDK
    Env {
        /// Walk directory tree to find project version
        #[arg(long)]
        detect: bool,

        /// Output as shell eval-able statements
        #[arg(long)]
        shell: bool,

        /// Print only JAVA_HOME path
        #[arg(long)]
        java_home_only: bool,
    },

    /// Shell integration management
    Shell {
        #[command(subcommand)]
        command: ShellCommands,
    },

    /// View or modify configuration
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },

    /// Diagnose common setup issues
    Doctor,

    /// Upgrade jm to the latest version
    Upgrade {
        /// Only check for updates, don't install
        #[arg(long)]
        check: bool,
    },
}

#[derive(Subcommand)]
pub enum ShellCommands {
    /// Print shell initialization script
    Init {
        /// Shell type (bash, zsh, fish, powershell)
        shell: String,
    },
    /// Print shell completion script
    Completions {
        /// Shell type (bash, zsh, fish, powershell)
        shell: String,
    },
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Show all config values
    List,
    /// Get a config value
    Get {
        /// Config key (e.g., global.preferred_distribution)
        key: String,
    },
    /// Set a config value
    Set {
        /// Config key
        key: String,
        /// Config value
        value: String,
    },
    /// Show config file path
    Path,
}
