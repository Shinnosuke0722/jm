# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- Pinned development, CI, release, and source builds to Rust 1.97.1 while
  keeping the workspace on Rust edition 2021.
- Migrated HTTP clients to Reqwest 0.13 with an explicit Rustls backend and
  platform-native certificate verification.

## [1.0.1] - 2026-08-09

### Added

- Added a committed lockfile so installs, CI, and release builds resolve the
  same dependency versions while preserving the declared Rust 1.82 MSRV.
- Added a Simplified Chinese README, focused Windows, project switching, and
  SDKMAN migration guides, plus issue and pull request templates, support
  guidance, a code of conduct, and Dependabot configuration.

### Changed

- Updated the canonical repository identity to `Shinnosuke0722/jm`.
- Reworked project documentation around verifiable behavior, platform limits,
  installation, project switching, and the supported `.sdkmanrc` Java entry.
- Modernized CI and release actions, locked dependency resolution, and added an
  explicit Rust 1.82 MSRV check.
- Made GitHub Release notes use the curated version section from this changelog.
- Limited ZIP extraction to the Stored and Deflate methods used by supported
  release archives, reducing optional dependency and security-audit surface.
- PowerShell auto-switching now updates `JAVA_HOME` and `PATH` natively, while
  Windows uses an NTFS junction for the active JDK without requiring Developer
  Mode or administrator privileges.

### Fixed

- Fixed the invalid CI workflow job name that prevented every job from starting.
- Wired `jm use --install` and `jm default --install` to install a missing JDK
  before selecting it.
- Fixed safe replacement and removal of the active Windows JDK junction, plus
  Windows extraction tests and unsupported ARM64 installer handling.
- Validated configured distribution names and provider-supplied filenames and
  Java version components before using them in filesystem paths.
- Made project, `use`, and `default` selection consistently choose the highest
  matching Java version, including numeric build components.
- Made release installers fail when a fetched checksum list omits the selected
  artifact instead of silently continuing without verification.
- Updated audited transitive dependencies while preserving the Rust 1.82 MSRV.

## [1.0.0] - 2026-04-05

### Added

- JDK installation, removal, local listing, remote search, and global default
  management.
- Project requirements through `.java-version`, plus reading the `java=` entry
  from `.sdkmanrc`.
- Shell initialization and completions for Bash, Zsh, Fish, and PowerShell.
- Foojay Disco package discovery with an Adoptium fallback for Temurin requests.
- Platform detection for Linux, macOS, and Windows on x86-64 and ARM64.
- GitHub Release builds for Linux x86-64/ARM64, macOS Intel/Apple silicon, and
  Windows x86-64.
- Configuration, diagnostics, proxy support, API caching, self-upgrade, and
  release installer scripts.

### Security

- Conditional SHA-256 verification for JDK downloads when the provider supplies
  checksum metadata.
- Exclusive locking and temporary-file replacement for registry writes.
- Cleanup of interrupted `.part` downloads.
- Validation of custom distribution names supplied in CLI version
  specifications before path and API use.

[Unreleased]: https://github.com/Shinnosuke0722/jm/compare/v1.0.1...HEAD
[1.0.1]: https://github.com/Shinnosuke0722/jm/releases/tag/v1.0.1
[1.0.0]: https://github.com/Shinnosuke0722/jm/releases/tag/v1.0.0
