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
java -version
```

`jm default` selects an installed match and updates the manager's global JDK link.

## 4. Pin Java for a project

From the project directory, resolve an installed JDK and write its full ID:

```sh
jm use 21
```

The command writes `.java-version`. Commit that file when the team should share
the same Java requirement.

## 5. Enable automatic switching

Add the command for your shell to its startup file:

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

The hook selects only an already installed JDK. It does not silently download a
missing project requirement.

## Continue

- Follow the complete [project switching rules](project-switching.md).
- Configure [Windows and PowerShell](windows.md).
- Move an existing Java entry with the [SDKMAN migration guide](sdkman-migration.md).
