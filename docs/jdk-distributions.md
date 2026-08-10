# Manage Temurin, Corretto, and GraalVM with jm

`jm` manages Eclipse Temurin, Amazon Corretto, GraalVM, and other JDK
distributions through one command-line workflow. You can install distributions
side by side, choose a global Java default, and pin a different JDK for each
project.

Package availability depends on the upstream catalog for your Java version,
operating system, and architecture. Search before installing when you need a
specific vendor build.

## Distribution names

Use these names in `jm install`, `jm search`, `jm list`, `jm use`, and `jm
default`:

| JDK distribution | jm input | Example |
| --- | --- | --- |
| Eclipse Temurin | `temurin` | `temurin-21` |
| Amazon Corretto | `corretto` | `corretto-17` |
| GraalVM Community Edition | `graalvm-ce` | `graalvm-ce-21` |
| Oracle GraalVM | `graalvm` | `graalvm-21` |

The GraalVM names are intentionally separate. Choose `graalvm-ce` for Community
Edition and `graalvm` for the Oracle distribution.

## Search the JDK catalog

Filter remote results by distribution and Java major version:

```sh
jm search 21 --distribution temurin
jm search 17 --distribution corretto
jm search 21 --distribution graalvm-ce
```

You can also list remote builds with explicit filters:

```sh
jm list --remote --distribution temurin --major 21
```

The catalog result is platform-specific. A command that returns a build on one
operating system or architecture may return no match on another.

## Install distributions side by side

```sh
jm install temurin-21
jm install corretto-17
jm install graalvm-ce-21
```

A distribution-qualified request prevents the configured preferred distribution
from changing the vendor you asked for. `jm install` resolves the latest
matching package unless the input includes a more specific Java version.

List the installed JDKs or inspect one distribution:

```sh
jm list
jm list --distribution temurin
```

## Choose the global Java default

Set the JDK used outside a project that has its own requirement:

```sh
jm default temurin-21
jm current
java -version
```

Changing the global default does not remove other installed distributions. To
make unqualified requests such as `jm install 21` prefer Corretto, change the
configuration:

```sh
jm config set global.preferred_distribution corretto
jm config get global.preferred_distribution
```

An explicit request such as `jm install temurin-21` still wins over that
preference.

## Pin a distribution per project

From a project directory, select an already installed JDK:

```sh
jm use graalvm-ce-21
```

`jm use` writes the resolved full installation ID to `.java-version`. Commit the
file when the project should share that exact requirement. The shell hook then
updates `JAVA_HOME` and `PATH` when you enter the directory.

For the complete detection order and missing-JDK behavior, read [Switch Java
versions per project](project-switching.md).

## Remove an old build

First run `jm list`, then uninstall the exact installation ID you no longer
need:

```sh
jm uninstall temurin-21.0.10+7
```

If the removed JDK was the global default, select another installed build with
`jm default <version>`.

## Provider behavior

`jm` queries the Foojay Disco catalog for JDK packages. Temurin requests can use
the Adoptium API as a fallback when the configured fallback is enabled; Corretto
and GraalVM requests do not use that Temurin-only fallback. Archive checksum
verification is performed when the provider supplies checksum metadata and
verification remains enabled.
