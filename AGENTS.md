# AGENTS.md — instructions for AI coding agents

This file tells AI coding agents (Cursor, Claude Code, Codex, Kimi CLI,
OpenCode, …) how to work **in this repository** and how to use Shehata Git
itself once it is installed.

## Working on the Shehata Git codebase

- Monorepo: `apps/desktop` (Tauri + React), `crates/*` (Rust), pnpm + Cargo.
- Business logic lives in Rust crates — never in Tauri handlers or React.
- Run before every commit:
  `cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && pnpm --filter @shehata/desktop typecheck && pnpm --filter @shehata/desktop test`
- Conventional Commits. Never commit secrets; the guard tests enforce it.
- Architectural change? Add an ADR under `docs/DECISIONS/`.
- Record what you actually did in `docs/BUILD_LOG.md`.

<!-- SHEHATA-GIT:START -->
## Git operations

Use the Shehata Git MCP tools for repository status, commits, pull, and push.

Before pushing:
1. Verify the assigned account.
2. Verify the remote and branch.
3. Never force push.
4. Do not change Git credentials.
5. Do not change the remote unless the user explicitly requests it.
<!-- SHEHATA-GIT:END -->
