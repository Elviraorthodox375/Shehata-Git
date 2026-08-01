# Security Policy

## Supported versions

| Version | Supported |
|---|---|
| 0.1.x   | ✅ (early development) |

## Reporting a vulnerability

**Please do not open a public issue for security reports.**

Email: security@shehata.dev (placeholder until the project domain is live —
until then, open a private GitHub security advisory on this repository).

Include: affected version, reproduction steps, impact. We acknowledge within
72 hours and coordinate disclosure with you.

## Scope notes

Shehata Git handles GitHub credentials indirectly (via the GitHub CLI) and
never stores them. Reports about token leakage through logs, database,
frontend state, or the MCP protocol are treated as **critical**.

The full security model lives in [docs/SECURITY.md](docs/SECURITY.md).
