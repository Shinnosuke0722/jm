# jm — JDK Manager

A fast, cross-platform CLI tool for installing and managing JDK versions.

- **Multi-distribution**: Install from Temurin, Corretto, Zulu, GraalVM, and 30+ other distributions
- **Cross-platform**: Linux, macOS, and Windows
- **Fast**: Near-zero shell overhead (~56ns per hook invocation, benchmarked)
- **Auto-switching**: Automatically switch JDK when entering a project directory

## Why jm?

| Feature | jm | SDKMAN | jabba | Coursier |
|---------|:--:|:------:|:-----:|:--------:|
| Multi-distribution (30+) | Yes | Yes | Limited | Limited |
| Cross-platform | Yes | Unix only | Yes | Yes |
| Shell auto-switching | Yes | Yes | Yes | No |
| No runtime dependency | Yes (static binary) | Requires bash+curl+zip | Requires Go | Requires JVM |
| Checksum verification | Yes (SHA256) | Partial | No | Yes |
| SDKMAN compatibility | Yes (.sdkmanrc) | - | No | No |
| Self-update | Yes | Yes | No | Yes |

## Quick Start

```bash
# Install JDK 21 (Temurin, the default distribution)
jm install 21

# Install a specific distribution
jm install corretto-17

# List installed JDKs
jm list

# Search available versions
jm search 21

# Switch JDK for the current project
jm use 21

# Set global default
jm default 21

# Show current JDK
jm current
```

## Installation

**Linux / macOS**

```bash
curl -fsSL https://raw.githubusercontent.com/lfming0419/jm/main/install.sh | sh
```

**Windows (PowerShell)**

```powershell
irm https://raw.githubusercontent.com/lfming0419/jm/main/install.ps1 | iex
```

**From Source**

```bash
cargo install --git https://github.com/lfming0419/jm.git
```

## Shell Integration

Add to your shell configuration:

```bash
# Bash (~/.bashrc)
eval "$(jm shell init bash)"

# Zsh (~/.zshrc)
eval "$(jm shell init zsh)"

# Fish (~/.config/fish/config.fish)
jm shell init fish | source

# PowerShell ($PROFILE)
jm shell init powershell | Invoke-Expression
```

This enables:
- Automatic `JAVA_HOME` and `PATH` configuration
- Auto-switching when you `cd` into a project with a `.java-version` file

## Project Configuration

Create a `.java-version` file in your project root:

```
temurin-21.0.10+7
```

Or just specify a major version:

```
21
```

`jm` also reads `.sdkmanrc` files for SDKMAN compatibility.

## Commands

| Command | Description |
|---------|-------------|
| `jm install <version>` | Install a JDK version |
| `jm uninstall <version>` | Remove an installed JDK |
| `jm list` | List installed JDKs |
| `jm list --remote` | List available remote versions |
| `jm search <query>` | Search available JDK packages |
| `jm use <version>` | Set JDK for current project |
| `jm default <version>` | Set global default JDK |
| `jm current` | Show currently active JDK |
| `jm which <binary>` | Show path to a JDK binary |
| `jm env` | Print JAVA_HOME/PATH settings |
| `jm shell init <shell>` | Print shell integration script |
| `jm config list\|get\|set` | View or modify configuration |
| `jm doctor` | Diagnose common setup issues |
| `jm upgrade` | Upgrade jm to the latest version |

## Supported Distributions

Powered by the [Foojay Disco API](https://foojay.io), `jm` supports 30+ JDK distributions including:

Eclipse Temurin, Amazon Corretto, Azul Zulu, Oracle JDK, GraalVM, BellSoft Liberica, SAP Machine, IBM Semeru, Microsoft OpenJDK, and more.

## Platform Support

| Platform | Architecture | Status |
|----------|-------------|--------|
| Linux | x86_64, aarch64 | Fully supported (musl static binary) |
| macOS | x86_64, aarch64 | Fully supported (universal binary) |
| Windows | x86_64 | Fully supported |

## Configuration

Configuration is stored in `config.toml` (run `jm config path` to find it).

```toml
[global]
preferred_distribution = "temurin"
auto_install = false

[install]
verify_checksum = true
keep_archives = false

[api]
fallback_enabled = true
timeout = 30
# proxy = "http://proxy:8080"
```

## Troubleshooting

**`jm` command not found after installation**

Ensure `~/.jm/bin` is in your PATH. The install script prints the exact command for your shell.

**Auto-switching not working**

Make sure shell integration is set up (see [Shell Integration](#shell-integration)). Run `jm doctor` to diagnose issues.

**Download fails behind a proxy**

Set the proxy in configuration:

```bash
jm config set api.proxy http://your-proxy:8080
```

**Checksum verification fails**

Try re-downloading with `jm install --no-verify <version>`. If the issue persists, the upstream archive may have been updated — please report it.

**`jm doctor` reports issues**

Run `jm doctor` for a full diagnostic. It checks: platform detection, storage directories, config, registry integrity, symlinks, installed JDK health, API connectivity, and shell integration.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup, project structure, and guidelines.

## Security

See [SECURITY.md](SECURITY.md) for vulnerability reporting and security practices.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.
