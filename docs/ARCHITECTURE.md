# Architecture

Shehata Git is a local-first monorepo. Every surface — desktop app, CLI,
credential helper, MCP server — calls the same Rust core, so behavior is
identical no matter where a Git operation starts.

## Layout

```text
apps/desktop          Tauri 2 + React + TypeScript + Tailwind (the window)
crates/
  shehata-core        Business logic: doctor, accounts, errors, redaction
  shehata-storage     SQLite (rusqlite), embedded migrations — never secrets
  shehata-git         System `git` runner (argument arrays only)
  shehata-github      System `gh` runner, just-in-time token retrieval
  shehata-cli         `shehata` binary
  shehata-credential-helper  `git-credential-shehata` binary
  shehata-mcp         `shehata-mcp` stdio MCP server (official rmcp SDK)
packages/             Shared JS (reserved)
docs/DECISIONS/       Architecture decision records
```

## Dependency direction

```text
Desktop UI ──Tauri commands──▶ shehata-core ──▶ shehata-storage
shehata-cli ─────────────────▶      │    └─────▶ shehata-git
shehata-mcp ─────────────────▶      └─────────▶ shehata-github
git-credential-shehata ──────▶ storage + github (read-only DB, JIT token)
```

Tauri command handlers and React components contain **no business logic**.

## Credential routing (the core idea)

1. Linking a repository writes a UUID marker into `<git-dir>/shehata-git/`
   and a row into SQLite (no tokens — ever).
2. Local (never global) git config points `credential.helper` at
   `shehata --repo-id <uuid>` after backing up previous values.
3. On any `git push`/`ls-remote`/fetch over HTTPS, git invokes
   `git-credential-shehata`, which looks up the assigned account and fetches a
   short-lived token via `gh auth token --user <login>` — held in memory only
   (`secrecy`), dropped immediately.
4. The GitHub CLI stays the credential source of truth. We never implement an
   OAuth app, never store tokens, never switch the active `gh` account.

## Safety model

- Fail closed: missing mapping/account/helper/db/token → no credentials.
- Force push, remote deletion, `reset --hard`, `clean -fd`, rebase, and amend
  are not implemented anywhere — not in UI, CLI, or MCP.
- All external processes run with argument arrays and timeouts.
- Audit log records actions, never secrets or file contents.

See `docs/DECISIONS/` for the reasoning behind each structural choice.
