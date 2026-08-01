# Security model

Shehata Git reduces wrong-account pushes without becoming another credential
store. Its default trust boundary is the local machine.

## Data boundaries

### Stored locally

- Canonical repository paths and safe remote metadata
- The selected account login and host for each repository
- Optional repository-local commit author name and email
- Exact backups of Git configuration values changed by Shehata Git
- Redacted action metadata and user preferences

### Never persisted by Shehata Git

- GitHub access tokens, passwords, cookies, or authorization headers
- Environment-variable dumps
- Repository source contents or diffs
- Credential-helper password output

SQLite includes a schema guard test that fails if a credential-shaped column is
introduced.

## Credential flow

```text
git push
  └─▶ repository-local credential.helper invokes git-credential-shehata
        ├─▶ resolves the repository UUID from a validated helper argument
        ├─▶ opens SQLite read-only and finds the exact assigned login
        ├─▶ verifies that the Git remote host matches the assignment
        ├─▶ requests a token from GitHub CLI for that exact host and login
        └─▶ returns it to Git over the credential protocol, then drops it
```

If any step fails, no credentials are emitted. The repository-local helper
configuration clears inherited helpers first, preventing silent fallback to a
different account.

## Process execution

- Commands launched by the application use explicit executables and argument
  arrays. Git's `!` credential-helper syntax is a required exception; its value
  is generated only from a canonical helper path and validated repository UUID.
- Hostnames, logins, repository IDs, and paths are validated before use.
- Commands have timeouts; background console programs use no-window flags on
  Windows.
- Raw GitHub CLI login output never crosses into the frontend. Only curated
  progress events and a validated one-time device code may cross Tauri IPC.

## Git configuration

Shehata Git modifies only repository-local configuration after review. Original
values are backed up before a change and restored exactly during unlink.
Repository operations never change the GitHub CLI default account. A user may
change that default through a separate, explicit, confirmed account action.

## Deliberately unavailable operations

Force push, hard reset, clean, rebase, amend, remote deletion, and arbitrary MCP
shell execution are not implemented.

## Logs, diagnostics, and MCP

Errors are passed through GitHub-token redaction before display. Activity events
contain summaries and outcomes, not tokens or file contents. Safe diagnostics
exclude account names, repository paths, remotes, environment values, and
credentials. MCP responses use structured envelopes and never return tokens.

## Vulnerability reporting

Follow the private process in the repository root [SECURITY.md](../SECURITY.md).
