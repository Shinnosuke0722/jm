# jm — JDK Manager

A fast, cross-platform CLI tool for installing and managing JDK versions.

- **Multi-distribution**: Install from Temurin, Corretto, Zulu, GraalVM, and 30+ other distributions
- **Cross-platform**: Linux, macOS, and Windows
- **Fast**: <10ms shell startup impact
- **Auto-switching**: Automatically switch JDK when entering a project directory

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
| `jm config path` | Show config file location |

## Supported Distributions

Powered by the [Foojay Disco API](https://foojay.io), `jm` supports 30+ JDK distributions including:

Eclipse Temurin, Amazon Corretto, Azul Zulu, Oracle JDK, GraalVM, BellSoft Liberica, SAP Machine, IBM Semeru, Microsoft OpenJDK, and more.

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

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.
