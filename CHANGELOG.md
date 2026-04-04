# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

## [1.0.0] - 2026-04-05

### Added
- Uninstall confirmation prompt (dialoguer), `--force` to skip
- HTTP proxy support threaded through all API clients and downloads
- Performance benchmarks confirming sub-microsecond operations
- Homebrew formula and Scoop manifest for package manager distribution
- CHANGELOG.md, CONTRIBUTING.md, SECURITY.md
- Feature comparison table and troubleshooting FAQ in README

### Security
- Registry: atomic writes (tmp+rename) and exclusive file locking (fs2)
- Symlink: atomic replacement via temp symlink + rename (eliminates TOCTOU)
- Download: PartFileGuard cleans up .part files on interruption
- Distribution: validate user-supplied names against `[a-zA-Z0-9_-]`

### Testing
- 26 CLI integration tests (assert_cmd)
- 7 API mock tests (wiremock)
- 9 shell crate tests
- 6 config load/save tests
- Criterion benchmarks (18 bench functions)
- Total: 89 tests, 0 clippy warnings

### CI/CD
- Swatinem/rust-cache on all CI and release jobs
- Security audit job (rustsec/audit-check)
- MSRV verification (Rust 1.82)
- Code coverage with cargo-llvm-cov and Codecov
- git-cliff for automated changelog generation

### Fixed
- Repository URLs corrected from `jm-sh/jm` to `lfming0419/jm`
- MSRV corrected from 1.75 to 1.82 (required by `is_none_or`)
- Code formatting normalized via `cargo fmt`

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

[Unreleased]: https://github.com/lfming0419/jm/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/lfming0419/jm/compare/v0.1.0...v1.0.0
[0.1.0]: https://github.com/lfming0419/jm/releases/tag/v0.1.0
