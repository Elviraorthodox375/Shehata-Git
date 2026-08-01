# Roadmap

> Status legend: ✅ done · 🚧 in progress · ⬜ planned

## Phase 0 — Environment & research ✅

- Toolchain verified/installed: Git 2.55, gh 2.97, Node 24, pnpm 9.15,
  Rust 1.97 (MSVC), VS Build Tools, WebView2.

## Phase 1 — Skeleton & branding ✅

- Monorepo: pnpm + Cargo workspaces, Tauri 2 + React 19 + strict TS.
- Brand: original S-node SVG mark, design tokens, app icons.
- All 7 Rust crates compile; shared error model with stable codes.
- Desktop window with full navigation (Home, Accounts, Repositories,
  AI Integration, Activity, Settings) + first-run onboarding.

## Phase 2 — Doctor ✅

- 8 live system checks (git, gh, accounts, database, helper, WebView2,
  PATH, MCP) with plain-language repair hints.
- `shehata doctor` CLI with `--json` and exit code 4 on unhealthy.

## Phase 3 — Accounts 🚧

- ✅ Real account cards from `gh auth status --json hosts`.
- ✅ Browser login (`gh auth login --web`) with progress modal.
- Avatar/profile metadata fetched token-in-backend only.
- Real owner browser-login acceptance is still pending.

## Phase 4 — Repositories 🚧

- ✅ Native folder picker and canonical worktree validation.
- ✅ Remote, branch, HEAD, upstream, status, identity, and local credential-config discovery.
- ✅ SQLite persistence that preserves existing routing state on refresh.
- ✅ Repository dashboard with branch, protocol, remote, and assignment state.
- Real user-repository selection acceptance is still pending.

## Phase 5 — Assignment & identity 🚧

- ✅ Assign an exact available account with host validation.
- ✅ Optional local-only `user.name` / `user.email` with original-value backups.
- ✅ Stable UUID marker under Git metadata with conflict protection.
- ✅ Desktop review-and-confirm assignment dialog.
- Real two-account/repository acceptance remains pending owner authentication.

## Phase 6 — Credential routing vertical slice 🚧

- ✅ Helper configured per repo with exact backup, verification, and rollback.
- ✅ Real `git credential fill` integration test through the built helper and assigned account mapping.
- ✅ Desktop Enable route, Verify (`git ls-remote`), and confirmed Unlink/Restore controls.
- Real owner `ls-remote`, external push, and two-account acceptance remain pending authentication.
- **First true product milestone.**

## Phase 7 — Safe Git actions 🚧

- ✅ Status and selected-path stage/unstage.
- ✅ Normal commit with conflict checks and audit events.
- ✅ Desktop Changes workflow.
- ✅ Pull `--ff-only` and normal push command paths.
- ✅ Full push preflight, non-fast-forward protection, policies, and audit events.
- ✅ Real local-remote pull/push regression test.
- Real GitHub acceptance remains pending authenticated owner repositories.

## Phase 8 — CLI 🚧

- ✅ All documented initial commands call shared core logic.
- ✅ Human output, structured `--json`, useful exit codes, and safe error output.
- ✅ CLI contract tests and fixed MCP launcher.
- Installer/PATH integration remains for Phase 10.

## Phase 9 — MCP 🚧

- ✅ Full reviewed 11-tool stdio surface calls shared core logic.
- ✅ Stable envelopes, strict repository inputs, and AI push-policy enforcement.
- ✅ Real protocol initialize/list/call contract test.
- ✅ Exact command/config UI and bounded AGENTS.md generation.
- External MCP Inspector/client acceptance remains for packaged binaries.

## Phase 10 — Quality & release 🚧

- ✅ Reproducible native sidecar build for the CLI, credential helper, and MCP server.
- ✅ NSIS installer bundles the desktop app and all three command-line binaries.
- ✅ Current-user PATH integration is added on install and removed on uninstall.
- ✅ Real silent install/run/uninstall smoke test passed on Windows x64.
- Test matrix completion, accessibility pass, hosted CI/draft release, code signing,
  and two-account manual acceptance remain.

## Explicitly out of scope for v0.1

GitLab/Bitbucket, PR/issue management, merge-conflict editor, cloud sync,
team credential sharing, mobile, macOS/Linux installers, custom OAuth app,
any destructive Git command, arbitrary shell execution via MCP.
