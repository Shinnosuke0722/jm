# Contributing to jm

Thank you for your interest in contributing to jm! This guide will help you get started.

## Development Setup

### Prerequisites

- Rust 1.82+ (see [rustup.rs](https://rustup.rs/))
- Git

### Building

```bash
git clone https://github.com/lfming0419/jm.git
cd jm
cargo build
```

### Running Tests

```bash
# All tests
cargo test --workspace

# Unit tests only
cargo test --workspace --lib

# Integration tests only
cargo test --test integration

# Benchmarks
cargo bench --bench benchmarks
```

### Linting

```bash
cargo fmt --all -- --check
cargo clippy --workspace --tests -- -D warnings
```

## Project Structure

```
jm/
├── src/                  # CLI binary
│   ├── main.rs           # Entry point
│   ├── cli.rs            # Clap argument definitions
│   ├── output.rs         # Formatting utilities
│   └── commands/         # Subcommand handlers
├── crates/
│   ├── jm-core/          # Core types: config, registry, version parsing, platform
│   ├── jm-api/           # API clients (Foojay Disco, Adoptium)
│   ├── jm-install/       # Download, extraction, verification, symlink
│   └── jm-shell/         # Shell integration scripts (bash, zsh, fish, powershell)
├── tests/integration/    # CLI integration tests (assert_cmd)
├── benches/              # Criterion benchmarks
├── install.sh            # Unix installer
└── install.ps1           # Windows installer
```

## Commit Convention

We use [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add support for Adoptium Temurin 23
fix: handle broken symlinks in jm doctor
test: add integration tests for config subcommands
ci: add MSRV check to CI pipeline
docs: update README with proxy configuration
refactor: extract HTTP client builder into shared helper
perf: cache parsed version specs
```

## Pull Request Process

1. Fork the repository and create a branch from `main`
2. Make your changes with clear, focused commits
3. Ensure all checks pass: `cargo fmt`, `cargo clippy`, `cargo test`
4. Open a PR against `main` with a clear description

## Adding a New Command

1. Create `src/commands/your_cmd.rs` with a `pub fn run(...) -> Result<()>`
2. Add the variant to `Commands` enum in `src/cli.rs`
3. Wire it up in `src/main.rs` match block
4. Add integration tests in `tests/integration/cli_test.rs`

## Adding a New Distribution

Known distributions are listed in `crates/jm-core/src/distribution.rs`. To add one:

1. Add a variant to the `Distribution` enum
2. Update `parse()`, `api_parameter()`, and `display_name()`
3. Add tests

Unknown distributions are automatically handled via `Distribution::Other(name)`.

## License

By contributing, you agree that your contributions will be licensed under the same terms as the project: MIT OR Apache-2.0.
