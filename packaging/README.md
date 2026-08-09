# Package-manager distribution

The `packaging/` directory contains the templates used to publish `jm` through
Homebrew, Scoop, and WinGet. A GitHub Release is the source of the versioned
archives and SHA-256 checksums; package-manager metadata must always reference an
immutable release tag.

## Release outputs

For every `v*` tag, `.github/workflows/release.yml` builds the platform archives,
generates `sha256sums.txt`, and attaches these package-manager files to the
release:

- `jm.rb` for `Shinnosuke0722/homebrew-tap`;
- `jm.json` for `Shinnosuke0722/scoop-bucket`;
- the three `Shinnosuke0722.jm*.yaml` WinGet manifests.

Never replace an archive or checksum under an existing tag. Publish a new version
instead, otherwise Homebrew, Scoop, and WinGet installations will fail integrity
checks.

## Homebrew

Copy the generated `jm.rb` to `Formula/jm.rb` in
`Shinnosuke0722/homebrew-tap`. Validate the tap on both macOS and Linux:

```sh
brew style --formula Shinnosuke0722/tap/jm
brew audit --strict --online Shinnosuke0722/tap/jm
brew install Shinnosuke0722/tap/jm
brew test Shinnosuke0722/tap/jm
jm --version
```

## Scoop

Copy the generated `jm.json` to `bucket/jm.json` in
`Shinnosuke0722/scoop-bucket`. Run the BucketTemplate checks and an x86-64
Windows installation smoke test:

```powershell
.\bin\checkver.ps1 jm
.\bin\checkurls.ps1 jm
.\bin\checkhashes.ps1 jm
.\bin\test.ps1

scoop bucket add shinnosuke0722 https://github.com/Shinnosuke0722/scoop-bucket
scoop install shinnosuke0722/jm
jm --version
```

The Windows x86-64 binary links against the Microsoft Visual C++ 2015-2022
runtime. The manifest suggests `extras/vcredist2022` for systems where that
runtime is not already installed.

## WinGet

Place the generated YAML files in the matching package-version directory in a
fork of `microsoft/winget-pkgs`. For version `1.0.2`, the directory is:

```text
manifests/s/Shinnosuke0722/jm/1.0.2/
```

Validate and test the complete multi-file manifest before submitting its own PR:

```powershell
winget validate --manifest .\manifests\s\Shinnosuke0722\jm\1.0.2
winget install --manifest .\manifests\s\Shinnosuke0722\jm\1.0.2 --scope user
jm --version
winget uninstall --id Shinnosuke0722.jm --exact
```

The WinGet manifest declares `Microsoft.VCRedist.2015+.x64` as a package
dependency. Do not mix documentation or unrelated versions into the WinGet PR.

## Package-managed upgrades

Package-manager installations must be upgraded through the manager that owns
the installation:

```text
Homebrew: brew upgrade Shinnosuke0722/tap/jm
Scoop:    scoop update jm
WinGet:   winget upgrade --id Shinnosuke0722.jm --exact
```

Do not use `jm upgrade` for a package-managed installation because it replaces
the binary directly and bypasses the package manager's version and hash records.
