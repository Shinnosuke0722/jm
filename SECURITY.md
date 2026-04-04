# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| latest  | Yes       |

## Reporting a Vulnerability

If you discover a security vulnerability in jm, please report it responsibly:

1. **Do NOT open a public GitHub issue**
2. Email the maintainer at [lfming0419@gmail.com](mailto:lfming0419@gmail.com)
3. Include a description of the vulnerability, steps to reproduce, and potential impact

## Response Timeline

- **Acknowledgment**: within 48 hours
- **Assessment**: within 7 days
- **Fix release**: within 30 days for critical issues

## Security Measures

jm implements the following security practices:

- **Checksum verification**: SHA256 checksums are verified on all JDK downloads by default
- **Atomic file operations**: Registry writes use temp-file + rename to prevent corruption
- **File locking**: Exclusive locks prevent concurrent process conflicts
- **Input validation**: Distribution names are restricted to `[a-zA-Z0-9_-]`
- **TLS**: All API and download connections use rustls (no OpenSSL dependency)
- **Dependency auditing**: CI runs `cargo audit` on every commit
- **No native code execution**: jm downloads and extracts archives but never executes downloaded binaries

## Scope

The following are in scope for security reports:

- Path traversal in archive extraction or version naming
- Arbitrary code execution
- Symlink attacks
- Privilege escalation
- Supply chain attacks (compromised dependencies)
- Man-in-the-middle attacks on download/API connections

The following are out of scope:

- Vulnerabilities in the JDK distributions themselves (report to the respective vendor)
- Denial of service via local resource exhaustion
