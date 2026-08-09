# Security Policy

## Supported Versions

Security fixes target the latest published release and the `main` branch. Older
release lines may not receive backports. The project does not currently promise
a fixed support window.

## Report a Vulnerability

Do not open a public issue for a suspected vulnerability.

Use [GitHub private vulnerability reporting](https://github.com/Shinnosuke0722/jm/security/advisories/new)
as the preferred channel. If that form is unavailable, email
[lfming0419@gmail.com](mailto:lfming0419@gmail.com).

Include:

- the affected jm version, operating system, and architecture;
- a description of the issue and its potential impact;
- minimal reproduction steps or a proof of concept;
- any suggested remediation or disclosure constraints.

Please avoid including unrelated secrets, personal data, or production
credentials. You may open a public issue only after the maintainer confirms that
coordinated disclosure is complete.

There is no guaranteed response SLA. The maintainer will make a reasonable
effort to acknowledge a complete report, assess impact, and coordinate a fix and
release before public disclosure.

## Relevant Security Behavior

- The default Foojay and Adoptium API endpoints use HTTPS through `reqwest`
  with rustls. The configurable Disco endpoint is trusted user configuration;
  jm does not currently reject a custom HTTP endpoint or a non-HTTPS archive
  URL returned by a provider.
- When verification is enabled and the selected provider supplies a SHA-256
  checksum, jm verifies the downloaded JDK archive before extraction. If the
  provider supplies no checksum, jm prints a warning and continues. Users can
  explicitly bypass verification with `jm install --no-verify`.
- Interrupted JDK downloads use a temporary `.part` file that is cleaned up on
  failure.
- Provider-supplied archive filenames and Java version components are validated
  before they are used to construct installation paths.
- Installed-JDK registry updates use an exclusive lock and a temporary-file
  replacement strategy.
- Configured and command-line custom distribution names are limited to ASCII
  letters, digits, `_`, and `-` before being used in paths or API parameters.
- The release workflow generates a `sha256sums.txt` file for published binary
  archives. Installer verification remains conditional on that file and the
  required local tooling being available.

These controls reduce risk but are not a guarantee that jm, its dependencies, or
third-party JDK archives are vulnerability-free.

## Report Scope

Examples of useful reports include:

- path traversal or unsafe archive extraction;
- arbitrary file overwrite or code execution;
- link or junction attacks during installation or default switching;
- registry corruption caused by concurrent operations;
- checksum bypasses or insecure transport behavior;
- unsafe self-upgrade behavior;
- vulnerabilities in a direct dependency that are exploitable through jm.

Vulnerabilities in a JDK distribution itself should also be reported to that
distribution's vendor. General support questions and non-security bugs belong in
the channels described in [SUPPORT.md](SUPPORT.md).
