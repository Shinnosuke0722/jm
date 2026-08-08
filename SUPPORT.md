# Support

Thanks for using jm. This project is maintained on a best-effort basis, but a
complete report helps the community respond quickly.

## Before Asking for Help

1. Read the [README](README.md), especially the installation, shell integration,
   and troubleshooting sections.
2. Search [existing issues](https://github.com/Shinnosuke0722/jm/issues) for the
   same behavior.
3. Update to the latest jm release and check whether the problem still occurs.
4. Collect the output of `jm --version`, `jm doctor`, and, when relevant,
   `java -version`.

Before sharing diagnostic output, remove credentials, tokens, proxy URLs,
private hostnames, usernames, and sensitive filesystem paths.

## Choose the Right Channel

- If jm crashes, returns an incorrect result, or behaves differently from the
  documentation, submit a
  [bug report](https://github.com/Shinnosuke0722/jm/issues/new?template=bug_report.yml).
- If you want to propose a new command, distribution, integration, or behavior,
  submit a
  [feature request](https://github.com/Shinnosuke0722/jm/issues/new?template=feature_request.yml).
- If the problem is caused by a downloaded JDK itself, contact that JDK
  distribution's vendor. jm manages JDK installations but does not maintain the
  JDK binaries.

Please do not use public issues for security vulnerabilities. Follow the private
reporting instructions in the [security policy](SECURITY.md) instead.

## What to Include

Useful support requests include:

- Operating system and version
- CPU architecture
- Shell and shell version
- jm version
- JDK distribution and requested version
- The exact command, sanitized output, and minimal reproduction steps
- What you expected and what happened instead

Screenshots can help with display problems, but paste searchable text whenever
possible. Maintainers may close reports that cannot be reproduced or that omit
information requested by the issue form.
