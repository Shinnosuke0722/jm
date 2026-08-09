# jm — Cross-platform JDK & Java Version Manager

[简体中文](README.zh-CN.md)

[![CI](https://github.com/Shinnosuke0722/jm/actions/workflows/ci.yml/badge.svg)](https://github.com/Shinnosuke0722/jm/actions/workflows/ci.yml)
[![Latest release](https://img.shields.io/github/v/release/Shinnosuke0722/jm)](https://github.com/Shinnosuke0722/jm/releases/latest)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

`jm` is a native Rust CLI for installing, switching, and pinning JDK versions on
Linux, macOS, and Windows. It resolves builds from multiple OpenJDK distributions,
sets a global Java default, and selects a project JDK from `.java-version` or the
`java=` entry in `.sdkmanrc`.

## Installation

### Linux and macOS

```sh
curl -fsSL https://raw.githubusercontent.com/Shinnosuke0722/jm/main/install.sh | sh
```

The installer downloads the matching asset from the latest GitHub Release into
`~/.jm/bin`. Review [`install.sh`](install.sh) before piping it to a shell if that
is your security policy.

### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/Shinnosuke0722/jm/main/install.ps1 | iex
```

The Windows installer adds `%USERPROFILE%\.jm\bin` to the user `PATH`. Open a new
terminal afterward, or follow the command printed by the installer. Prebuilt
Windows releases currently target x86-64; see the [Windows guide](docs/windows.md)
for PowerShell setup and ARM64 notes.

### Build from source

Requires Rust 1.82 or newer:

```sh
cargo install --git https://github.com/Shinnosuke0722/jm.git --locked
```

Release installers attempt to verify the downloaded `jm` archive against the
published SHA-256 list when the required checksum data and local tooling are
available.

## Quick Start

```sh
# Install the latest matching Temurin JDK 21
jm install 21

# Install another distribution
jm install corretto-17

# Select the global default
jm default 21

# Pin the latest installed JDK 21 for this project
jm use 21

# Inspect the result
jm current
java -version
```

`jm use 21` resolves an already installed match and writes its full installation
ID to `.java-version`. Commit that file when everyone on the project should use
the same JDK requirement.

## Why jm?

- **One CLI across desktop platforms.** The same install, search, list, default,
  and project-pin workflow is available on Linux, macOS, and Windows.
- **Multiple JDK distributions.** Query the [Foojay Disco API](https://foojay.io/)
  for Temurin, Corretto, Zulu, Liberica, Microsoft OpenJDK, GraalVM, and other
  distributions exposed for your platform. Temurin requests can fall back to the
  Adoptium API when Disco is unavailable.
- **Project-aware switching.** Shell hooks walk from the current directory toward
  the filesystem root and select an installed JDK from `JM_JAVA_VERSION`,
  `.java-version`, or the Java entry in `.sdkmanrc`.
- **No JVM required to run the manager.** `jm` itself is a compiled Rust command-
  line program.
- **Integrity checks when metadata exists.** JDK archives are checked against a
  provider-supplied SHA-256 value when verification is enabled and that value is
  available. If the provider supplies no checksum, `jm` warns and continues.

## Shell Integration

Add the command for your shell to its startup file:

```sh
# Bash (~/.bashrc)
eval "$(jm shell init bash)"

# Zsh (~/.zshrc)
eval "$(jm shell init zsh)"

# Fish (~/.config/fish/config.fish)
jm shell init fish | source
```

```powershell
# PowerShell ($PROFILE)
jm shell init powershell | Invoke-Expression
```

The hook updates `JAVA_HOME` and places the selected JDK's `bin` directory on
`PATH` for the current shell. A matching JDK must already be installed; project
detection does not silently download one.

For precedence rules, file examples, and missing-version behavior, read
[Project JDK switching](docs/project-switching.md).

## Project Configuration

The recommended project file is `.java-version`:

```text
temurin-21.0.10+7
```

A major version such as `21` is also accepted, although a full installed ID is
more reproducible. `jm use <version>` writes the resolved full ID automatically.

If a project already uses SDKMAN, `jm` can read the `java=` entry:

```properties
java=21.0.2-tem
kotlin=2.1.0
```

Only `java=` is interpreted. `jm` does not install other SDKMAN candidates,
execute SDKMAN hooks, or claim full `.sdkmanrc` compatibility. See
[Migrating a Java project from SDKMAN](docs/sdkman-migration.md).

## Supported Platforms

The release workflow is configured for these artifacts:

| Operating system | Architecture | Release artifact |
| --- | --- | --- |
| Linux | x86-64 | `jm-linux-x86_64.tar.gz` |
| Linux | ARM64 | `jm-linux-aarch64.tar.gz` |
| macOS | Intel and Apple silicon | Per-architecture archives plus a universal archive |
| Windows | x86-64 | `jm-windows-x86_64.zip` |

The CLI recognizes Linux, macOS, and Windows on x86-64 or ARM64. Windows ARM64
does not currently receive a prebuilt release artifact and is best-effort when
built from source.

Package availability still depends on the selected distribution, Java version,
operating system, architecture, and the upstream provider catalog.

## Distribution Names

Common names and aliases include:

| Distribution | Example input |
| --- | --- |
| Eclipse Temurin | `temurin-21` |
| Amazon Corretto | `corretto-17` |
| Azul Zulu | `zulu-21` |
| BellSoft Liberica | `liberica-21` |
| Microsoft OpenJDK | `microsoft-21` |
| GraalVM Community Edition | `graalvm-ce-21` |

Use `jm search <distribution>` or `jm list --remote --major <major>` to inspect
what the provider currently exposes for your platform.

## Commands

| Command | Purpose |
| --- | --- |
| `jm install <version>` | Download and install a matching JDK |
| `jm uninstall <version>` | Remove an installed JDK |
| `jm list` | List installed JDKs |
| `jm list --remote` | List maintained remote Java versions |
| `jm search <query>` | Search remote JDK packages |
| `jm use <version>` | Write a project `.java-version` from an installed match |
| `jm default [version]` | Set or display the global default JDK |
| `jm current` | Explain the active project requirement or global default |
| `jm which [binary]` | Print a binary path from the global default JDK |
| `jm env` | Print `JAVA_HOME` and `PATH` values |
| `jm shell init <shell>` | Print a shell integration script |
| `jm shell completions <shell>` | Generate shell completions |
| `jm config list\|get\|set\|path` | Inspect or edit configuration |
| `jm doctor` | Run local configuration and connectivity checks |
| `jm upgrade` | Upgrade from the latest GitHub Release |

Run `jm <command> --help` for all flags and examples.

## Configuration and Troubleshooting

Use `jm config path` to locate `config.toml` and `jm config list` to inspect the
effective values. Common settings include the preferred distribution, proxy,
Adoptium fallback, and archive retention.

```sh
jm config set global.preferred_distribution zulu
jm config set api.proxy http://proxy.example:8080
jm doctor
```

Notes:

- `jm install --no-verify <version>` explicitly bypasses JDK archive checksum
  verification. Use it only after evaluating the download source.
- If a project requirement is not installed, `jm current` reports it and the
  shell hook keeps using the global default until a match exists.
- Homebrew and Scoop files in `packaging/` are release templates. There is no
  official tap or bucket advertised by this repository yet.
- `jm upgrade` requires a published GitHub Release with an asset matching the
  current operating system and architecture.

## Documentation

- [Windows and PowerShell](docs/windows.md)
- [Project JDK switching](docs/project-switching.md)
- [SDKMAN Java migration](docs/sdkman-migration.md)
- [Contributing](CONTRIBUTING.md)
- [Security policy](SECURITY.md)
- [Changelog](CHANGELOG.md)

## Contributing

Bug reports, documentation fixes, and focused pull requests are welcome. Read
[CONTRIBUTING.md](CONTRIBUTING.md) before making a change. For usage questions,
see [SUPPORT.md](SUPPORT.md).

## License

Licensed under either the [Apache License, Version 2.0](LICENSE-APACHE) or the
[MIT License](LICENSE-MIT), at your option.
