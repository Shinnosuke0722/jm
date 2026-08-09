---
title: Get started with jm
description: Install jm, choose a JDK distribution, set a global Java default, and pin a reproducible JDK version for a project.
---

# Get started with jm

`jm` installs and selects JDK builds without requiring a JVM to run the manager itself.
This guide takes you from a fresh shell to a project with a reproducible Java
requirement.

## 1. Install jm

::: code-group

```sh [Linux and macOS]
curl -fsSL https://raw.githubusercontent.com/Shinnosuke0722/jm/main/install.sh | sh
```

```powershell [Windows PowerShell]
irm https://raw.githubusercontent.com/Shinnosuke0722/jm/main/install.ps1 | iex
```

```sh [Build from source]
cargo install --git https://github.com/Shinnosuke0722/jm.git --locked
```

:::

The source build requires Rust 1.97.1 or newer. Prebuilt GitHub Release users do
not need Rust or Cargo.

Before continuing, make sure the current terminal can find `jm`. Choose the PATH
command that matches the installation method and shell:

::: code-group

```sh [Prebuilt — Linux and macOS]
export PATH="$HOME/.jm/bin:$PATH"
```

```powershell [Prebuilt — PowerShell]
$env:Path = "$HOME\.jm\bin;$env:Path"
```

```sh [Source — Linux and macOS]
export PATH="${CARGO_INSTALL_ROOT:-${CARGO_HOME:-$HOME/.cargo}}/bin:$PATH"
```

```powershell [Source — PowerShell]
$cargoRoot = if ($env:CARGO_INSTALL_ROOT) {
  $env:CARGO_INSTALL_ROOT
} elseif ($env:CARGO_HOME) {
  $env:CARGO_HOME
} else {
  "$HOME\.cargo"
}
$env:Path = "$cargoRoot\bin;$env:Path"
```

:::

These commands activate the default locations for the current session. Follow
the prebuilt installer's printed instructions to persist its directory, or open
a new terminal after the Windows installer updates the user PATH. If you chose a
custom prebuilt directory or configured Cargo's `--root`, `install.root`,
`CARGO_INSTALL_ROOT`, or `CARGO_HOME`, substitute the corresponding installation
root. Verify the CLI is available before moving to step 2:

```sh
jm --version
```

## 2. Install a JDK

Ask for a Java major version or include a distribution name:

```sh
jm install 21
jm install corretto-17
```

The available result still depends on the upstream provider catalog for your
operating system and architecture.

## 3. Set the global default

```sh
jm default 21
jm current
```

`jm default` selects an installed match and updates the manager's global JDK link.

## 4. Enable automatic switching

Run the command for your shell now, then add the same line to its startup file:

::: code-group

```sh [Bash]
eval "$(jm shell init bash)"
```

```sh [Zsh]
eval "$(jm shell init zsh)"
```

```fish [Fish]
jm shell init fish | source
```

```powershell [PowerShell]
jm shell init powershell | Invoke-Expression
```

:::

The command initializes the current shell, setting `JAVA_HOME` and putting the
selected JDK's `bin` directory on `PATH`. Verify that the shell now uses the
selected JDK:

```sh
java -version
```

The hook selects only an already installed JDK. It does not silently download a
missing project requirement.

## 5. Pin Java for a project

From the project directory, resolve an installed JDK and write its full ID:

```sh
jm use 21
```

The command writes `.java-version`. Commit that file when the team should share
the same Java requirement.

## Continue

- Follow the complete [project switching rules](project-switching.md).
- Configure [Windows and PowerShell](windows.md).
- Move an existing Java entry with the [SDKMAN migration guide](sdkman-migration.md).
