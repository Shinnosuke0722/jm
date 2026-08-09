# Use jm as a Java version manager on Windows

`jm` supports Windows with PowerShell integration, project-level JDK selection,
and a user-scoped installation directory. The published Windows release artifact
currently targets x86-64.

## Install on Windows x86-64

Run in PowerShell:

```powershell
irm https://raw.githubusercontent.com/Shinnosuke0722/jm/main/install.ps1 | iex
```

The installer downloads `jm-windows-x86_64.zip` from the latest GitHub Release,
attempts to verify it against the release SHA-256 list, installs `jm.exe` under
`%USERPROFILE%\.jm\bin`, and adds that directory to the user `PATH`.

Open a new PowerShell window after installation. To check discovery:

```powershell
Get-Command jm
jm --version
```

If your policy does not permit piping remote content to `Invoke-Expression`,
download and inspect
[`install.ps1`](https://github.com/Shinnosuke0722/jm/blob/main/install.ps1) before
running it locally.

## Build from source

Install Rust 1.97.1 or newer with the MSVC toolchain, then run:

```powershell
cargo install --git https://github.com/Shinnosuke0722/jm.git --locked
```

Rust normally places the binary in `%USERPROFILE%\.cargo\bin`.

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
operating system. Use `jm config path` and `jm doctor` to see resolved paths.

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

Open a new terminal first. Then confirm the user `PATH` contains
`%USERPROFILE%\.jm\bin`:

```powershell
[Environment]::GetEnvironmentVariable("Path", "User")
```

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
