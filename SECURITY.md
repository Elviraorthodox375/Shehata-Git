# Security Policy

## Supported versions

Shehata Git is currently an early preview. Security fixes are applied to the
latest `0.1.x` source and the newest published preview, when one exists.

| Version | Supported |
|---|---|
| Latest `0.1.x` | Yes |
| Older previews | No |

## Report a vulnerability privately

Do **not** open a public issue and do not include tokens, credentials, private
repository URLs, or customer information in a report.

Use GitHub's private vulnerability reporting for this repository:

<https://github.com/moshehata95/Shehata-Git/security/advisories/new>

Include the affected version or commit, impact, minimum reproduction steps, and
whether the issue may have exposed credential material. Maintainers will aim to
acknowledge complete reports within 72 hours and will coordinate remediation
and disclosure with the reporter.

## High-priority areas

- Token or credential exposure through logs, SQLite, UI state, diagnostics,
  errors, clipboard behavior, or MCP responses
- Routing a repository to an identity other than the one explicitly assigned
- Command injection, unsafe path handling, or arbitrary shell execution
- Repository-local Git configuration changes without exact backup and restore
- Bypassing push policies or gaining access to destructive Git operations

The complete design and trust boundaries are documented in
[docs/SECURITY.md](docs/SECURITY.md).
