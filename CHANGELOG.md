# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

### CI/CD
- Add Swatinem/rust-cache to all CI and release jobs
- Add security audit job (rustsec/audit-check)
- Add MSRV verification (Rust 1.75)
- Add code coverage with cargo-llvm-cov and Codecov
- Add git-cliff for automated changelog generation

### Testing
- Add 26 CLI integration tests (assert_cmd)
- Add 7 API mock tests (wiremock)
- Add 9 shell crate tests
- Add 6 config load/save tests
- Add criterion benchmarks (18 bench functions)

### Security
- Registry: atomic writes (tmp+rename) and exclusive file locking (fs2)
- Symlink: atomic replacement via temp symlink + rename (eliminates TOCTOU)
- Download: PartFileGuard cleans up .part files on interruption
- Distribution: validate user-supplied names against `[a-zA-Z0-9_-]`

### Added
- Uninstall confirmation prompt (dialoguer), `--force` to skip
- HTTP proxy support threaded through all API clients and downloads
- Performance benchmarks confirming sub-microsecond operations

### Fixed
- Repository URLs corrected from `jm-sh/jm` to `lfming0419/jm`

## [0.1.0] - 2026-04-05

### Added
- Core CLI with 13 commands: install, uninstall, list, search, use, default, current, which, env, shell, config, doctor, upgrade
- Multi-distribution support (13+ distributions) via Foojay Disco API
- Adoptium API fallback when Disco is unavailable
- Cross-platform support: Linux (x86_64, aarch64), macOS (x86_64, aarch64, universal), Windows (x86_64)
- Shell integration with auto-switching for bash, zsh, fish, PowerShell
- Project-level version detection via `.java-version` and `.sdkmanrc` files
- SHA256 checksum verification on download
- API response caching with configurable TTL
- TOML-based configuration system
- Self-update mechanism (`jm upgrade`)
- One-command install scripts for Unix (`install.sh`) and Windows (`install.ps1`)
- CI pipeline with check, format, clippy, and cross-platform tests
- Multi-platform release pipeline with universal macOS binary
- Dual license: MIT OR Apache-2.0

[Unreleased]: https://github.com/lfming0419/jm/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/lfming0419/jm/releases/tag/v0.1.0
