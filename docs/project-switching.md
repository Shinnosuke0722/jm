# Project JDK switching with jm

`jm` can select an installed JDK as you move between Java projects. The selected
home is exposed through `JAVA_HOME`, and its `bin` directory is placed on the
current shell's `PATH`.

## 1. Enable the shell hook {#enable-the-shell-hook}

Add one initialization command to the shell startup file:

```sh
# Bash: ~/.bashrc
eval "$(jm shell init bash)"

# Zsh: ~/.zshrc
eval "$(jm shell init zsh)"

# Fish: ~/.config/fish/config.fish
jm shell init fish | source
```

```powershell
# PowerShell: $PROFILE
jm shell init powershell | Invoke-Expression
```

Restart the shell after editing its startup file. The generated hook runs once
at startup and again when the shell observes a directory change (or, in
PowerShell, before rendering the prompt).

## 2. Install and pin a JDK

`jm use` only selects from installed JDKs. Install the requirement first:

```sh
jm install temurin-21
cd path/to/project
jm use temurin-21
```

The second command resolves the latest matching installed build and writes its
full ID to `.java-version`, for example:

```text
temurin-21.0.10+7
```

Commit `.java-version` when the requirement should be shared by the project.
Team members still install their own matching JDK; the file does not contain a
downloaded runtime.

You can also create the file manually. A broad requirement such as `21` works,
but a distribution-qualified or full ID is less ambiguous.

## Detection order

For project-aware commands, `jm` resolves the requirement as follows:

1. `JM_JAVA_VERSION`, if it is set and non-empty.
2. Starting at the current directory, walk toward the filesystem root. At each
   directory, check `.java-version` first and then `.sdkmanrc`.
3. If no installed project match is available, the shell environment falls back
   to the global default selected with `jm default`.

This means the nearest project file wins. `.java-version` takes precedence over
`.sdkmanrc` only when both files are in the same directory. The environment
variable overrides all project files.

Example temporary override:

```sh
export JM_JAVA_VERSION=corretto-17
```

```powershell
$env:JM_JAVA_VERSION = "corretto-17"
```

Unset the variable to return to file-based detection.

## What happens when the JDK is missing?

Project detection does not silently install a runtime. `jm current` displays the
requested version and warns when no installed JDK matches it. The shell hook then
keeps the global default active until you install a match:

```sh
jm current
jm install temurin-21
```

If several installed builds match a broad requirement, `jm` selects the newest
according to its parsed Java version ordering. Use `jm use` to write the resolved
full ID when reproducibility matters.

## Global default versus project requirement

Use the global default outside pinned projects:

```sh
jm default 21
jm default
```

`jm default` updates the `current` directory link in jm's data directory. A
project requirement does not rewrite that global link; the shell hook adjusts
the current process environment instead.

## Verify the active selection

```sh
jm current
java -version
```

`jm current` explains where the requirement came from. `java -version` confirms
which runtime the current shell actually executes. If they disagree, restart the
shell or trigger another directory change, then run `jm doctor`.

## SDKMAN projects

`jm` reads only the `java=` entry in `.sdkmanrc`. Other candidates and SDKMAN
environment behavior are outside its scope. See
[Migrating a Java project from SDKMAN](sdkman-migration.md) for supported suffixes
and a safe migration path.
