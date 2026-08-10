# macOS competitive benchmark — preliminary evidence (2026-08-11)

## Status

This is a reproducible **preliminary** macOS result, not evidence for a public
cross-platform performance claim. The protocol review gates did not pass:

- Homebrew Hyperfine 1.20.0 rejected the documented `--random-order` option, so
  the tool samples were not automatically interleaved.
- The prescribed javm fixture layout was not recognised by javm 0.13.2: each
  local-list validation produced only its header and zero JDK rows. Its timing
  samples are retained but it is excluded from local-list ranking.
- Cold-remote and controlled trusted-HTTPS installation cases were not run. In
  particular, no local CA was added to the macOS keychain and no installation
  ranking is claimed.
- The run was recorded on battery power (67% charge), although Low Power Mode
  and thermal-pressure warnings were off. A plugged-in, quiet-machine rerun is
  required before publication.

The raw data is intentionally retained outside the production site in
[`results/`](../results), with a matching archive and SHA-256 checksum.

## Environment and versions

| Item | Value |
|---|---|
| Machine | MacBookPro17,1, Apple M1 (arm64), 8 cores, 8 GiB RAM |
| OS | macOS 26.5.2 (25F84) |
| Translation | `sysctl.proc_translated = 0` |
| Hyperfine | 1.20.0 |
| jm | 1.0.2 |
| jabba | 0.15.0 |
| javm | 0.13.2 |
| mise | 2026.8.3 |
| SDKMAN observation | script 5.23.0, native 0.7.34, Homebrew Bash 5.3.15 |

All four release assets were official Apple Silicon artifacts and matched the
pinned SHA-256 values in [the macOS protocol](benchmarking-macos.md). The
extracted executable hashes, artifact hashes, `file` output, and versions are
in `results/`.

## Native startup and peak RSS

Each native `--version` command used five warmups and 50 formal Hyperfine
samples. Peak RSS was measured independently with `/usr/bin/time -l`; 20 raw
samples were retained per native tool (two ten-sample passes, neither dropped).

| Tool | Median (ms) | P95 (ms) | Median peak RSS (MiB) | P95 peak RSS (MiB) |
|---|---:|---:|---:|---:|
| **jm** | **4.063** | **6.076** | **7.156** | **7.188** |
| javm | 5.312 | 5.743 | 11.664 | 11.812 |
| mise | 7.091 | 7.487 | 13.500 | 13.578 |
| jabba | 39.736 | 40.683 | 13.734 | 14.062 |

`--help` and invalid-command startup samples were also captured (40 each).
Their raw distributions are in `startup-help-native.json` and
`startup-invalid-native.json`; invalid-command runs used Hyperfine's explicit
`--ignore-failure` flag.

SDKMAN was added as requested, but is intentionally not in the native table:
the clean-Bash `sdk version` observation had a 1,023.682 ms median and
1,069.529 ms P95 over 50 samples. This includes Bash plus SDKMAN client
initialization, so it is a user-experience observation rather than a native CLI
comparison.

## Local installed-JDK scaling

The validation files prove that jm, jabba, and mise recognised exactly
0/10/100/1000 entries (jm adds four table framing lines). javm reported no JDK
entries and SDKMAN has no equivalent local-list operation; both are excluded
from this table. Each cell is median/P95 milliseconds.

| Installed JDKs | jm | jabba | mise |
|---:|---:|---:|---:|
| 0 | **6.449 / 8.699** | 42.253 / 43.771 | 10.479 / 11.193 |
| 10 | **6.662 / 6.994** | 41.867 / 42.999 | 10.923 / 11.419 |
| 100 | **7.281 / 7.530** | 42.500 / 44.477 | 13.685 / 14.127 |
| 1,000 | **11.441 / 11.879** | 46.655 / 48.344 | 41.631 / 43.437 |

No outlier was removed. The jabba 10-entry distribution includes a 97.2 ms
sample and Hyperfine's warning; the raw JSON is retained unchanged.

## Eight-process local-list concurrency

Two warmup batches and 12 formal batches launched eight 100-entry list
processes simultaneously. javm was excluded because its fixture validation
failed; SDKMAN has no equivalent local-list command. The results below come
from `concurrency-n100.json` and include the batch completion time plus
`8 / median_batch_seconds` throughput.

| Tool | Median batch (ms) | P95 batch (ms) | Median throughput (commands/s) |
|---|---:|---:|---:|
| **jm** | **16.224** | **17.673** | **493.09** |
| mise | 27.807 | 30.457 | 287.70 |
| jabba | 81.831 | 85.068 | 97.76 |

## Project-resolution depth

Only jm and mise were ranked because both traverse the parent project tree in
this setup. Values are median/P95 milliseconds from 25 samples after five
warmups.

| Depth | jm `env --detect` | mise `hook-env` |
|---:|---:|---:|
| 0 | **6.132 / 7.640** | 13.019 / 13.589 |
| 5 | **5.917 / 6.573** | 13.803 / 14.922 |
| 20 | **6.113 / 6.791** | 15.289 / 15.563 |
| 50 | **6.239 / 7.106** | 21.859 / 23.544 |
| 100 | **6.400 / 7.058** | 44.701 / 45.966 |

## Artifact retention and integrity

The following artifacts must remain together:

- raw directory: [`results/`](../results)
- archive: [`jm-macos-benchmark-results.tar.gz`](../jm-macos-benchmark-results.tar.gz)
- SHA-256 file: [`jm-macos-benchmark-results.tar.gz.sha256`](../jm-macos-benchmark-results.tar.gz.sha256)

Archive SHA-256: `421bd434ba11336129a8e596794cc226b30d9c331bcbd4dd4500d12942180b3b`.
It was verified with `shasum -a 256 -c jm-macos-benchmark-results.tar.gz.sha256`.

The machine record excludes serial number, hardware UUID, and provisioning
UDID; these do not affect reproducibility and should not be published.
