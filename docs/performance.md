# jm performance: Windows benchmark evidence

`jm` is designed to keep common local JDK-management paths small and direct. A
Windows x64 comparison measured process startup, peak working set, local JDK
listing, project resolution, shell initialization, concurrency, binary size,
and cold remote queries against jabba, javm, and mise.

## What the benchmark supports

On the tested Windows x64 machine:

- common local commands completed with **30% to 46% lower median latency** than
  the fastest competitor in each matching scenario;
- `jm --version` completed in **27.721 ms**, 42.7% below the fastest competing
  result of 48.415 ms;
- `jm --version` used a **6.52 MiB peak working set**, 15.3% below the
  lowest-memory competing result of 7.70 MiB; and
- the release build measured **3.97 MiB**, 46.6% smaller than the next-smallest
  binary in the comparison.

These claims apply to the measured Windows x64 environment and tool versions.
They do not claim that jm wins every command, operating system, or workload.

## Startup latency and memory

Median wall-clock time and peak working set for a new `--version` process:

| Tool | Version | Median time | P95 time | Peak working set |
|---|---:|---:|---:|---:|
| **jm** | 1.0.2 | **27.721 ms** | **30.662 ms** | **6.52 MiB** |
| mise | 2026.8.3 | 48.415 ms | 53.469 ms | 13.59 MiB |
| jabba | 0.15.0 | 49.224 ms | 54.596 ms | 7.70 MiB |
| javm | 0.13.2 | 49.527 ms | 55.771 ms | 8.14 MiB |

The same ordering held for `--help` and an invalid command. `jm` measured
28.201 ms and 28.207 ms respectively, 38.8% and 37.9% below the fastest
competing result for each scenario.

## Local list scaling

Median time to list isolated local JDK fixtures:

| Installed JDKs | jm | jabba | javm | mise | jm versus fastest competitor |
|---:|---:|---:|---:|---:|---:|
| 0 | **28.795 ms** | 53.808 ms | 61.388 ms | 56.796 ms | 46.5% lower |
| 10 | **29.038 ms** | 53.987 ms | 61.029 ms | 62.147 ms | 46.2% lower |
| 100 | **29.804 ms** | 53.767 ms | 63.329 ms | 108.869 ms | 44.6% lower |
| 1,000 | **40.195 ms** | 57.493 ms | 85.561 ms | 569.017 ms | 30.1% lower |

At 1,000 entries, jabba used less peak working-set memory than jm: 7.93 MiB
versus 8.79 MiB. That is why the evidence supports a lower startup-memory
claim, not an "always lowest memory" claim.

## Other local paths

| Scenario | jm | Next-fastest comparable result |
|---|---:|---:|
| Current active JDK | **28.780 ms** | javm, 49.266 ms |
| Resolve JDK home | **28.502 ms** | jabba, 49.255 ms |
| Generate shell initialization | **28.064 ms** | javm, 50.222 ms |
| Eight concurrent 100-entry list commands | **83.17 commands/s** | jabba, 74.16 commands/s |

For parent-directory project detection, jm moved from 28.993 ms at the project
root to 30.516 ms at depth 50. The comparable mise results were 69.633 ms and
203.020 ms. jabba and javm only read their project file in the current
directory, so they were not ranked for parent traversal.

## Where jm did not lead

Cold, isolated remote Temurin 21 queries were an observational network test,
not a measure of CLI startup alone:

| Tool | Median time | Returned bytes |
|---|---:|---:|
| **javm** | **942.303 ms** | 111 |
| jm | 1,084.221 ms | 509 |
| mise | 2,032.968 ms | 308 |
| jabba | 2,446.847 ms | 8 |

The upstream APIs, response sizes, and formatting differ, so this result must
not be presented as a controlled backend benchmark.

The Windows run also did not produce a valid installation ranking. A shared
public JDK download was interrupted by a CDN TLS timeout, while a local HTTP
fixture was rejected by javm's HTTPS-only download policy. No security check or
competitor binary was modified to manufacture a result.

## Method and limits

- Platform: Windows 10 x64, Intel Core i7-6700K, 16 GiB RAM, balanced power plan.
- Build under test: commit `5ccc0108ced112915323c3ca6b20d483321b042a`.
- Every formal timing sample created a new process and captured redirected
  output as part of the measured command.
- Core cases used five warmups followed by randomized, interleaved formal runs.
- `--version` used 50 formal samples per tool. Other local cases used 10 to 40
  samples according to cost; network queries used five fresh-cache samples.
- Peak working set came from Windows `GetProcessMemoryInfo`.
- Tool homes, caches, configuration, and local JDK fixtures were isolated.
- Default outputs are not identical. The benchmark measures the real default
  command experience, not normalized work per output byte.
- mise is a broader, multi-language tool; its results show end-user command
  cost, not equal-scope implementation efficiency.

The complete Chinese-language audit contains all tables, hashes, sample counts,
side effects, and interpretation limits:
[Windows benchmark report](https://github.com/Shinnosuke0722/jm/blob/main/docs/performance-benchmark-2026-08-11.md).

To add macOS evidence before treating these results as cross-platform, follow
the [macOS competitive benchmark protocol](benchmarking-macos.md). Keep the
Windows and macOS results separate until both data sets pass the same review.
