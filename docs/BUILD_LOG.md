# Build log

This file records verified engineering milestones without machine-specific
paths, account names, repository names, credentials, or private test data.

## 2026-08-01 — Public repository preparation

- Reworked the README, roadmap, contribution guide, security policy, changelog,
  community templates, and CI workflow for an open-source release.
- Removed stale internal handoff material and sanitized historical build notes.
- Added branded screenshot placeholders so real screenshots can be reviewed for
  personal data before publication.
- Clarified GitHub CLI default-account behavior in the UI and CLI.
- Added an explicit, confirmed default-account switch that never changes
  repository assignments.
- Refined search focus styling across account, repository, picker, and activity
  surfaces.
- Removed executable paths from the copyable diagnostic report and added a
  regression test for that privacy boundary.
- Fixed clean-checkout CI ordering so Tauri sidecars are built before Rust
  workspace validation.
- Verified a dependency-free working-tree copy: frozen pnpm install, sidecar
  release build, frontend lint/typecheck/tests/production build, Rust format,
  Clippy with warnings denied, and all workspace tests passed.
- Built the current Windows NSIS installer locally after the public-release
  changes. The unsigned artifact remains local and unpublished.

## 2026-08-01 — v0.1.6 Smart Sync workspace polish

- Replaced silent disabled sync controls with a remote, identity, and route
  readiness checklist plus a direct setup action.
- Consolidated Smart Sync to one primary action and replaced its native push
  prompt with the shared Liquid Glass confirmation dialog.
- Added working-tree search, staged/changed/untracked filters, visible-file
  selection, and human-readable Windows paths.
- Replaced the remaining native confirmation prompts for setup, unlink, and
  normal push with consistent in-app dialogs.
- Added unit coverage for workspace filtering and Windows path display.
- Passed the full Rust and frontend quality gate, built the Windows NSIS
  installer, installed v0.1.6, and verified the installed sidebar version,
  repository workspace, Smart Sync readiness guard, search/filter controls,
  and in-app default-account confirmation flow.

## 2026-08-01 — v0.1.5 desktop workflow refinement

- Adopted the official Shehata Git logo across in-app and bundle assets.
- Added scalable searchable selectors, list search, and collapsed repository
  cards.
- Added real browser-login cancellation and repeat-copy confirmation.
- Added local audit-event deletion and full history clearing.
- Reduced repeated UI fetching and parallelized independent repository route
  checks with a bounded concurrency limit.
- Built and smoke-tested the Windows NSIS installer locally. No artifact was
  published.

## 2026-07-31 — Initial product foundation

- Created the Tauri/React desktop, shared Rust core, storage, Git/GitHub runners,
  CLI, credential helper, and MCP server.
- Implemented system diagnostics, account discovery, repository registration,
  identity assignment, credential routing, safe Git actions, audit events, and
  configuration backup/restore.
- Added unit, integration, protocol-contract, and security guard tests using
  temporary repositories and fake binaries.
