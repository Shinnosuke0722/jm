# Migrate a Java project from SDKMAN to jm

`jm` offers a narrow interoperability path for Java projects that already have a
`.sdkmanrc`: it reads the first non-comment `java=<version>` entry while walking
up from the current directory.

This is not full SDKMAN compatibility. `jm` does not install or switch Kotlin,
Maven, Gradle, or other SDKMAN candidates, does not run `sdk env`, and does not
consume SDKMAN's local installation catalog.

## Supported Java entry

Given this file:

```properties
java=21.0.2-tem
kotlin=2.1.0
```

`jm` interprets the Java requirement as Temurin 21.0.2 and ignores the Kotlin
line. The parser expects the exact `java=` key form; `java = ...` is not treated
as a Java entry.

Recognized SDKMAN vendor suffix mappings include:

| SDKMAN suffix | jm distribution |
| --- | --- |
| `tem` | `temurin` |
| `amzn` | `corretto` |
| `zulu` | `zulu` |
| `oracle` | `oracle` |
| `librca` | `liberica` |
| `sapmchn` | `sapmachine` |
| `sem` | `semeru` |
| `graalce` | `graalvm-ce` |
| `graal` | `graalvm` |
| `ms` | `microsoft` |
| `mandrel` | `mandrel` |

Unknown suffixes are passed to the Foojay lookup as distribution names after
normal validation. Availability depends on the upstream catalog and platform.

## Safe migration workflow

1. Inspect the project's Java entry.

   ```sh
   grep '^java=' .sdkmanrc
   ```

   In PowerShell:

   ```powershell
   Select-String -Path .sdkmanrc -Pattern '^java='
   ```

2. Install the corresponding JDK using jm's distribution-first syntax. For
   `java=21.0.2-tem`, use:

   ```sh
   jm install temurin-21.0.2
   ```

3. Enable the [jm shell hook](project-switching.md#1-enable-the-shell-hook), enter
   the project, and verify both the detected requirement and running Java:

   ```sh
   jm current
   java -version
   ```

4. Keep `.sdkmanrc` if SDKMAN users or other SDK candidates still depend on it.
   `jm` and SDKMAN should not both own the same shell's Java switching hook at the
   same time; choose one manager per active shell to avoid competing `PATH` and
   `JAVA_HOME` changes.

5. If the project is moving fully to jm, write a dedicated file:

   ```sh
   jm use temurin-21.0.2
   ```

   This creates `.java-version` with the full installed ID. Once the team has
   agreed on the migration, remove only the obsolete Java entry—or the whole
   `.sdkmanrc` if no other SDKMAN candidates use it.

## Resolution details

- When `.java-version` and `.sdkmanrc` are in the same directory,
  `.java-version` wins.
- A nearer `.sdkmanrc` can win over a `.java-version` located in a parent
  directory because jm searches directory by directory.
- The suffix mapping expresses a distribution and Java version requirement; it
  does not guarantee the exact same archive or build identity that SDKMAN used.
- Missing matches are not installed automatically. Run `jm install` explicitly.
