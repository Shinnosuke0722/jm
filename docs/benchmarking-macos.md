# Run the competitive benchmark on macOS

Use this protocol to add macOS evidence to the existing Windows comparison. It
is intentionally stricter than timing `--version`: the matrix covers startup,
memory, local-state scaling, project resolution, shell initialization,
concurrency, remote queries, and a controlled installation pipeline.

Do not merge macOS numbers into the public cross-platform claim until the run
passes the validation and review gates at the end of this guide.

## 1. Freeze the comparison

Use the same versions as the Windows run so operating-system differences are
not mixed with release differences:

| Tool | Version |
|---|---:|
| jm | 1.0.2 |
| jabba | 0.15.0 |
| javm | 0.13.2 |
| mise | 2026.8.3 |

Also record SDKMAN's `sdk version` when it is included as a shell-client
observation. SDKMAN is not a pinned native executable: its official client is
loaded into Bash. Keep it in a separate column and do not use it to rank the
native process-startup results above.

Run all four as native binaries for the same architecture. Do not compare an
Apple Silicon binary with an x86-64 binary running under Rosetta.

```sh
uname -m
sysctl -in sysctl.proc_translated 2>/dev/null || true
```

Expected architecture values are `arm64` or `x86_64`; the translation check
must be absent or `0`.

Download official release artifacts into an isolated directory rather than
using package-manager shims. The release pages are the source of truth:

- [jm 1.0.2](https://github.com/Shinnosuke0722/jm/releases/tag/v1.0.2)
- [jabba 0.15.0](https://github.com/Jabba-Team/jabba/releases/tag/0.15.0)
- [javm 0.13.2](https://github.com/felipebz/javm/releases/tag/v0.13.2)
- [mise 2026.8.3](https://github.com/jdx/mise/releases/tag/v2026.8.3)

Use these native assets:

| Architecture | jm | jabba | javm | mise |
|---|---|---|---|---|
| Apple Silicon | `jm-macos-aarch64.tar.gz` | `jabba-0.15.0-darwin-arm64` | `javm-darwin-arm64.tar.gz` | `mise-v2026.8.3-macos-arm64` |
| Intel | `jm-macos-x86_64.tar.gz` | `jabba-0.15.0-darwin-amd64` | `javm-darwin-x86_64.tar.gz` | `mise-v2026.8.3-macos-x64` |

Verify every artifact against its release SHA-256 before extracting it. Record
the hash of the final executable too:

```sh
shasum -a 256 downloads/*
shasum -a 256 bin/jm bin/jabba bin/javm bin/mise
file bin/jm bin/jabba bin/javm bin/mise
bin/jm --version
bin/jabba --version
bin/javm --version
bin/mise --version
```

Pinned SHA-256 values from those release assets:

| Architecture | jm | jabba | javm | mise |
|---|---|---|---|---|
| Apple Silicon | `13e72b77b8ed8fcbd655c68351fcc931e5765b2ab72d8bc9710f1a79cb925356` | `6dab7523cfc2eb34eff877df2534c09a652390db9b6c3b9105e9f7b461c4229e` | `fe2f9946c5f23431d617063d9ae3bc3de2670f919443ea6987ae3dc5fc5b25e1` | `e2f25fc3a2fe82f15c33a2c7a2ec4cc0ed09eb9a1edc5cbcfe2f7f9902bfa4af` |
| Intel | `9f19b8280b2596fb36f2ca5350e4234aed361b9747930e61aca5201110b51b86` | `dbc469be6ed31e40a05904f82cc062e87b1fbff0ecb8007a6ce14502457b5c33` | `e1182edfa58f535532b584eb94b253c68f27fec26f72f7012a80c0d003ccac53` | `2bc0c80e8a7a33e545490dcfe73a73f20dfa964e62c237f296b34eff6c55a2d2` |

Do not copy a digest across architectures. Recheck the release metadata before
the run and record any mismatch instead of silently replacing the pinned
binary.

## 2. Prepare a quiet machine

Use a dedicated benchmark directory and a plugged-in Mac. Disable Low Power
Mode, close developer tools and sync-heavy applications, and wait for indexing
or backups to finish. Do not run with `sudo` except where the controlled HTTPS
certificate setup explicitly requires it.

Record the machine state before testing:

```sh
mkdir -p results
{
  date -u
  sw_vers
  uname -a
  system_profiler SPHardwareDataType
  sysctl -n hw.memsize
  sysctl -n hw.ncpu
  sysctl -n hw.physicalcpu
  sysctl -n hw.logicalcpu
  pmset -g batt
  pmset -g custom
} > results/machine.txt
```

Run a short dry run, then confirm the machine is not thermally constrained:

```sh
pmset -g therm
```

If macOS reports CPU, GPU, or thermal pressure, cool the machine and restart the
whole formal run. Do not keep only the faster half of a throttled run.

Install the timing tool and record its version:

```sh
brew install hyperfine
hyperfine --version | tee results/hyperfine-version.txt
python3 --version | tee results/python-version.txt
```

## 3. Use isolated state

Create one benchmark root and keep all tool-owned state below it:

```sh
export BENCH_ROOT="$PWD/.benchmark-work"
mkdir -p "$BENCH_ROOT"/{bin,fixtures,results,tmp}
export NO_COLOR=1
export TERM=dumb
export TMPDIR="$BENCH_ROOT/tmp"
```

For every tool and fixture size, set independent homes:

```sh
export JM_HOME="$BENCH_ROOT/fixtures/n100/jm"
export JABBA_HOME="$BENCH_ROOT/fixtures/n100/jabba"
export JAVM_HOME="$BENCH_ROOT/fixtures/n100/javm"
export MISE_DATA_DIR="$BENCH_ROOT/fixtures/n100/mise-data"
export MISE_CONFIG_DIR="$BENCH_ROOT/fixtures/n100/mise-config"
export MISE_CACHE_DIR="$BENCH_ROOT/fixtures/n100/mise-cache"
export MISE_STATE_DIR="$BENCH_ROOT/fixtures/n100/mise-state"
export MISE_NO_AUTO_INSTALL=1
export SDKMAN_DIR="$BENCH_ROOT/fixtures/n100/sdkman"
```

Generate equivalent fixture counts of 0, 10, 100, and 1,000. Each fake JDK must
contain an executable `bin/java` file; javm fixtures also need a `release` file
with `JAVA_VERSION`, `JAVA_VENDOR`, and `OS_ARCH`. jm's `registry.json` must
contain the same number of registered installations. mise installations live
under `installs/java/VERSION`, and jabba/javm installations use
`jdk/temurin@VERSION`.

For SDKMAN, copy a complete, isolated SDKMAN installation into each fixture and
place fake JDKs under `candidates/java/VERSION-tem`; point its `current` link at
one such candidate. Do not source a user's `~/.sdkman` or profile files. SDKMAN
requires Bash 4 or later, so record the Bash path and version used by the run.

The following generator creates that layout plus the project-depth fixtures:

```sh
python3 <<'PY'
import json
import os
import platform
from pathlib import Path

root = Path(os.environ["BENCH_ROOT"]).resolve()
java_arch = "aarch64" if platform.machine() == "arm64" else "x86_64"

def write_jdk(path: Path, version: str, release: bool = False) -> None:
    binary = path / "bin" / "java"
    binary.parent.mkdir(parents=True, exist_ok=True)
    binary.write_text("#!/bin/sh\nexit 0\n", encoding="ascii")
    binary.chmod(0o755)
    if release:
        (path / "release").write_text(
            f'JAVA_VERSION="{version}"\n'
            'JAVA_VENDOR="Eclipse Adoptium"\n'
            f'OS_ARCH="{java_arch}"\n',
            encoding="ascii",
        )

for scale in (0, 10, 100, 1000):
    fixture = root / "fixtures" / f"n{scale}"
    jm_home = fixture / "jm"
    jabba_home = fixture / "jabba"
    javm_home = fixture / "javm"
    mise_data = fixture / "mise-data"
    for path in (
        jm_home,
        jabba_home / "jdk",
        javm_home / "jdk",
        mise_data / "installs" / "java",
        fixture / "cwd",
    ):
        path.mkdir(parents=True, exist_ok=True)

    installations = []
    for index in range(1, scale + 1):
        version = f"21.0.{index}"
        identifier = f"temurin-{version}"
        jm_path = jm_home / "jdks" / identifier
        write_jdk(jm_path, version)
        installations.append({
            "id": identifier,
            "distribution": "temurin",
            "java_version": {
                "major": 21,
                "minor": 0,
                "patch": index,
                "build": None,
            },
            "full_version": version,
            "major_version": 21,
            "path": str(jm_path),
            "installed_at": "2026-08-11T00:00:00Z",
            "is_lts": True,
        })
        write_jdk(jabba_home / "jdk" / f"temurin@{version}", version)
        write_jdk(javm_home / "jdk" / f"temurin@{version}", version, True)
        write_jdk(mise_data / "installs" / "java" / version, version)

    (jm_home / "registry.json").write_text(
        json.dumps({"format_version": 1, "installations": installations}),
        encoding="utf-8",
    )
    autodiscover = javm_home / "autodiscover"
    autodiscover.mkdir(parents=True, exist_ok=True)
    (autodiscover / "config.json").write_text(json.dumps({
        "enabled": True,
        "sources": {
            "system": False,
            "jabba": False,
            "gradle": False,
            "intellij": False,
            "javm": True,
        },
        "cache_ttl": 86400000000000,
    }), encoding="utf-8")

current = root / "fixtures" / "n10" / "jm" / "current"
current.symlink_to(root / "fixtures" / "n10" / "jm" / "jdks" / "temurin-21.0.1")

for depth in (0, 5, 20, 50, 100):
    project = root / "fixtures" / "projects" / f"d{depth}"
    project.mkdir(parents=True, exist_ok=True)
    (project / ".java-version").write_text("21.0.1\n", encoding="ascii")
    (project / ".jabbarc").write_text("temurin@21.0.1\n", encoding="ascii")
    (project / "mise.toml").write_text('[tools]\njava = "21.0.1"\n', encoding="ascii")
    leaf = project
    for _ in range(depth):
        leaf /= "d"
    leaf.mkdir(parents=True, exist_ok=True)
    (project / "leaf.txt").write_text(str(leaf), encoding="utf-8")
PY
```

Run the generator only inside the disposable benchmark root. If rerunning in
the same directory, delete that exact fixture root first so a previous cache,
symlink, or partial install cannot contaminate the next run.

Validate every list command before timing it. The number of recognized entries
must equal the requested fixture scale. Save this validation output; a fast
empty result is a failed fixture, not a benchmark win.

Warm javm's ordinary discovery cache once for its normal-list test. Also keep a
separate first-discovery scenario that deletes only javm's generated discovery
cache before every sample.

## 4. Measure wall-clock latency

Use direct binaries and disable Hyperfine's shell so shell startup is not added
to the CLI result. Use five warmups and interleave tools with `--random-order`.
The following example covers `--version`; repeat the pattern for every row in
the matrix below.

```sh
hyperfine \
  --shell=none \
  --warmup 5 \
  --runs 50 \
  --random-order \
  --export-json results/startup-version.json \
  "$BENCH_ROOT/bin/jm --version" \
  "$BENCH_ROOT/bin/jabba --version" \
  "$BENCH_ROOT/bin/javm --version" \
  "$BENCH_ROOT/bin/mise --version"
```

Hyperfine commands that intentionally fail need `--ignore-failure`. Verify their
exit codes separately before the formal run.

Use these minimum formal sample counts:

| Scenario | Runs per tool |
|---|---:|
| `--version` | 50 |
| `--help`, invalid command | 40 each |
| local list: 0 and 10 entries | 30 each |
| local list: 100 entries | 20 |
| local list: 1,000 entries | 10 |
| current JDK, resolve JDK home, shell init | 30 each |
| project resolution at each depth | 25 |
| first discovery | 10 |
| eight-process concurrent batches | 12 |
| cold remote query | 5 |
| controlled install | 10 after 1 warmup |

Report median and P95. Do not rank by the minimum sample.

## 5. Measure peak resident memory separately

Do not infer memory from Hyperfine. Run at least ten additional samples per
scenario through macOS `/usr/bin/time -l` and extract `maximum resident set
size`, which is reported in bytes:

```sh
for run in $(seq 1 10); do
  /usr/bin/time -l "$BENCH_ROOT/bin/jm" --version \
    >/dev/null 2>>results/jm-version-memory.txt
done

grep 'maximum resident set size' results/jm-version-memory.txt
```

Calculate the median and P95 in bytes, then convert to MiB by dividing by
1,048,576. Keep timing and memory samples as separate distributions; the
`time` wrapper adds overhead and must not replace direct wall-clock samples.

Also record binary sizes:

```sh
stat -f '%N,%z' "$BENCH_ROOT/bin/"{jm,jabba,javm,mise} \
  > results/binary-sizes.csv
```

## 6. Run the complete local matrix

Use the nearest equivalent command for each tool and document differences:

| Scenario | jm | jabba | javm | mise |
|---|---|---|---|---|
| Startup | `--version`, `--help`, invalid command | same | same | same |
| Local list | `list --no-color` | `ls` | `ls` | `ls --installed java --no-header` |
| Current JDK | `current --no-color` | `current` | `current` | not ranked unless an equivalent PATH-current command is identified |
| Resolve home | `which java --no-color` | `which 21` | `which 21` | `where java@21.0.1` |
| Project resolve | `env --detect --shell --no-color` | `--fd3 /dev/null use` at depth 0 only | same | `hook-env --force --quiet --shell zsh` |
| Shell init | `shell init zsh` | exclude if it writes a profile | `init zsh` | `activate zsh` |

SDKMAN is an additional, shell-client observation: run it through a clean Bash
that sources only the isolated `SDKMAN_DIR/bin/sdkman-init.sh`, then use `sdk
version`, `sdk current java`, and `sdk home java VERSION-tem` where configured.
Its startup includes Bash and SDKMAN initialization, and it has no local-list
command with the same semantics as the native tools, so exclude it from native
startup and local-list rankings. `sdk list java` is a remote catalogue query and
may be retained only as an observational network sample.

Create project trees at depths 0, 5, 20, 50, and 100. Put `.java-version`,
`.jabbarc`, and `mise.toml` at the root. Only rank tools at depths they actually
traverse; a tool that reads only the current directory has not completed the
same work.

For concurrency, start eight 100-entry list processes at the same instant,
wait for all eight, and measure batch completion. Run two warmup batches and 12
formal batches. Report batch median and `8 / batch_seconds` throughput.

Capture stdout byte counts for every case. Default list output differs greatly
between tools, so the final report must disclose output volume next to scaling
results.

## 7. Keep remote queries observational

Use a brand-new isolated home/cache for every remote sample and rotate tool
order. Query Temurin 21:

```sh
jm list --remote --distribution temurin --major 21 --no-color
jabba ls-remote 21 --latest major
javm ls-remote 21 --distribution temurin --latest major
mise ls-remote java@temurin-21
```

Record elapsed time, exit code, response lines, and response bytes. Do not call
this a controlled backend benchmark because the tools may use different APIs,
filters, caches, and response models.

## 8. Run installation through trusted local HTTPS

Installation needs its own controlled test. Public CDN timings are too noisy,
and javm correctly rejects plain HTTP.

1. Create a deterministic ZIP containing a top-level JDK directory with
   executable `bin/java`, a `release` file, and at least 32 MiB of deterministic
   payload data.
2. Use `mkcert` or another local CA to issue a `localhost` certificate trusted
   by the macOS System Keychain. Record the certificate fingerprint.
3. Run a local HTTPS service that exposes Foojay-compatible `/packages` and
   `/ids/PACKAGE_ID` JSON plus the same ZIP at `/fixture.zip`.
4. Configure jm's `api.disco_api_url` and `JAVM_DISCO_API` to that HTTPS base.
   Give jabba the same `zip+https://localhost/.../fixture.zip` URL.
5. Use a fresh destination and tool home for every sample. Run one warmup and
   ten randomized formal samples.
6. Measure and report metadata lookup, time to first byte, download, checksum,
   extraction, registration/default-link work, total wall time, CPU time, peak
   RSS, written bytes, and file count separately where the tool permits it.
7. Validate that every tool extracted the same payload and that `bin/java`
   exists. A successful exit without the expected payload is a failed sample.
8. Remove the local CA and certificate from the Keychain after the run, stop
   the server, and verify both are gone.

Do not disable TLS verification or patch a competitor to accept HTTP. If all
three clients cannot trust the same local certificate, mark installation as
inconclusive instead of publishing a ranking.

The comparable commands are:

```sh
# jm: config.toml points api.disco_api_url at the mock Foojay service.
JM_HOME="$RUN_ROOT/jm" jm install temurin-21.0.1 --quiet --no-color

# jabba: the exact same ZIP bypasses vendor discovery but not HTTPS transport.
JABBA_HOME="$RUN_ROOT/jabba" jabba install \
  "21.0.1=zip+https://localhost:PORT/fixture.zip" \
  --output "$RUN_ROOT/jabba-output"

# javm: the mock Foojay service returns the exact same ZIP and checksum.
JAVM_HOME="$RUN_ROOT/javm" \
JAVM_DISCO_API="https://localhost:PORT" \
  javm install temurin@21.0.1 --output "$RUN_ROOT/javm-output"
```

mise's normal Java backend cannot be assumed to accept that same custom
Foojay fixture. Unless its documented backend can be configured to consume the
identical artifact without patching mise, exclude it from the shared-artifact
ranking. You may run `mise install java@temurin-21.0.1` as a separate native
end-to-end observation, but label it as non-comparable because discovery and
download infrastructure differ.

For a `mkcert` setup, record the generated CA location and leaf fingerprint,
then use `mkcert -uninstall` after the test. Confirm removal with Keychain
Access or `security find-certificate`; do not leave the benchmark CA trusted.

## 9. Validate and package the evidence

Before comparing results:

- every case must have its expected sample count and exit status;
- fixture validation must show 0, 10, 100, and 1,000 recognized entries;
- no formal result may be silently dropped as an outlier;
- thermal pressure must remain clear;
- tool versions, executable hashes, machine metadata, stdout sizes, raw timing
  JSON, and raw memory logs must be present; and
- the SDKMAN script/native version, Bash version, and isolated SDKMAN_DIR must
  be present if SDKMAN observations are included; and
- the benchmark directory must not contain credentials, ordinary user config,
  or unrelated shell profiles.

Create a results archive before cleanup:

```sh
tar -czf jm-macos-benchmark-results.tar.gz results
shasum -a 256 jm-macos-benchmark-results.tar.gz \
  > jm-macos-benchmark-results.tar.gz.sha256
```

Keep raw machine-readable results outside the production site. Commit only the
reviewed summary and the archive checksum unless the repository deliberately
adopts a benchmark-data directory.

## 10. Review gates before production

The macOS evidence is ready to update the public claim only when:

1. another reviewer can recompute every median and P95 from the raw files;
2. command semantics and output differences are documented;
3. installation uses the same trusted HTTPS service and artifact;
4. Apple Silicon and Intel results are not combined unless both were run;
5. Windows and macOS tables remain labeled separately; and
6. the homepage claim still describes the weakest supported result, not the
   strongest isolated number.

After review, update the [performance evidence page](performance.md), keep the
benchmark date and versions visible, run the documentation checks, and only
then promote the draft claim to production.
