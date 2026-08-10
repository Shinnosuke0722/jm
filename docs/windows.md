# JDK version manager for Windows

`jm` is a native JDK version manager for Windows. It installs multiple Java
Development Kits, selects a global default, and switches Java versions per
project through PowerShell. The manager and its JDK data stay in user-scoped
locations, so a normal setup does not require administrator privileges.

The published Windows release artifact currently targets x86-64. If you are
moving from SDKMAN, read the [SDKMAN alternative on Windows](sdkman-migration.md)
guide after installing `jm`.

## Install with Scoop on Windows x86-64

```powershell
scoop bucket add shinnosuke0722 https://github.com/Shinnosuke0722/scoop-bucket
scoop install shinnosuke0722/jm
```

The Scoop manifest suggests `extras/vcredist2022` if the Microsoft Visual C++
2015-2022 runtime is not already installed.

## Install with WinGet on Windows x86-64 after upstream acceptance

The first manifest is under review in
[`microsoft/winget-pkgs#414637`](https://github.com/microsoft/winget-pkgs/pull/414637).
Once it appears in the WinGet community source, install it with:

```powershell
winget install --id Shinnosuke0722.jm --exact
```

Confirm availability first with `winget search --id Shinnosuke0722.jm --exact`.
The WinGet manifest installs the required Microsoft Visual C++ runtime as a
package dependency.

## Install with the PowerShell release script

Run in PowerShell:

```powershell
irm https://raw.githubusercontent.com/Shinnosuke0722/jm/main/install.ps1 | iex
```

The installer downloads `jm-windows-x86_64.zip` from the latest GitHub Release,
attempts to verify it against the release SHA-256 list, installs `jm.exe` under
`%USERPROFILE%\.jm\bin`, and adds that directory to the user `PATH`.

The current x86-64 binary links against the Microsoft Visual C++ 2015-2022
runtime. If a direct installation reports that `VCRUNTIME140.dll` is missing,
install Microsoft's
[latest supported Visual C++ Redistributable](https://learn.microsoft.com/cpp/windows/latest-supported-vc-redist).

Open a new PowerShell window after installation. To check discovery:

```powershell
Get-Command jm
jm --version
```

If your policy does not permit piping remote content to `Invoke-Expression`,
download and inspect
[`install.ps1`](https://github.com/Shinnosuke0722/jm/blob/main/install.ps1) before
running it locally.

## Upgrade jm

Use the manager that owns the installation:

```powershell
# Scoop
scoop update jm

# WinGet
winget upgrade --id Shinnosuke0722.jm --exact
```

Do not run `jm upgrade` for a Scoop- or WinGet-managed installation because it
replaces the binary directly and bypasses the package manager's version and hash
records. `jm upgrade` remains available for the direct PowerShell release-script
installation.

## Build from source

Install Rust 1.97.1 or newer with the MSVC toolchain, then run:

```powershell
cargo install --git https://github.com/Shinnosuke0722/jm.git --locked
```

Rust normally places the binary in `%USERPROFILE%\.cargo\bin`.
Upgrade that source installation by re-running the command with `--force`:

```powershell
cargo install --git https://github.com/Shinnosuke0722/jm.git --locked --force
```

The CLI recognizes Windows ARM64, but the release workflow does not currently
publish an ARM64 Windows archive. Building that target from source is best-effort
and is not covered by the prebuilt-release guarantee.

## Enable PowerShell switching

Create the profile if needed:

```powershell
if (!(Test-Path $PROFILE)) {
    New-Item -ItemType File -Path $PROFILE -Force | Out-Null
}
notepad $PROFILE
```

Add this line and restart PowerShell:

```powershell
jm shell init powershell | Invoke-Expression
```

The hook checks the current directory before each prompt, updates `JAVA_HOME`,
and puts the selected JDK's `bin` directory on the current process `PATH`.

## Install and select Java

```powershell
jm install 21
jm default 21

Set-Location C:\path\to\project
jm use 21

jm current
java -version
```

`jm use` writes `.java-version`; it does not download a missing runtime. Read
[Project JDK switching](project-switching.md) for precedence and fallback rules.

## Directory-link behavior

The global default is represented by a `current` directory link inside jm's data
directory. On Windows, jm creates a directory junction, so a normal user-level
setup does not require Developer Mode or administrator privileges solely for jm.

If link creation still fails, run:

```powershell
jm doctor
```

Then check whether endpoint security software or filesystem policy blocks links
under the data directory. Do not run the whole shell as Administrator unless your
organization's policy specifically requires it.

## Data location

By default, jm follows the Windows application-data locations exposed by the
operating system. Use `jm config path` to print the `config.toml` path. The
storage-directory check in `jm doctor` reports the resolved data directory.

To keep data, configuration, and cache under one explicit root, set `JM_HOME`
before invoking jm or initializing its hook:

```powershell
$env:JM_HOME = "$env:USERPROFILE\.jm-data"
jm config path
```

Put the assignment before the `jm shell init powershell` line in `$PROFILE` if
you want it applied to every PowerShell session.

## Troubleshooting

### `jm` is not found after installation

Open a new terminal first, then run `Get-Command jm`. Check the location that
matches the installation method:

```powershell
# Scoop
scoop prefix jm
scoop checkup

# WinGet
winget list --id Shinnosuke0722.jm --exact
Test-Path "$env:LOCALAPPDATA\Microsoft\WinGet\Links\jm.exe"

# PowerShell release installer
Test-Path "$env:USERPROFILE\.jm\bin\jm.exe"

# Cargo source installation
Test-Path "$env:USERPROFILE\.cargo\bin\jm.exe"
```

If the expected executable exists but is not discovered, inspect the user
`PATH` with `[Environment]::GetEnvironmentVariable("Path", "User")` and repair
the corresponding Scoop shim, WinGet Links, `.jm\bin`, or `.cargo\bin` entry.

### The project version is detected but Java does not change

```powershell
jm current
Get-Command java
$env:JAVA_HOME
jm doctor
```

Confirm the required JDK is installed with `jm list`, and confirm the profile
contains the initialization line only once.

### A profile or script is blocked

Inspect the effective policies with `Get-ExecutionPolicy -List` and follow your
organization's approved policy. Avoid weakening machine-wide execution policy as
a workaround.
