# Changelog

All notable changes to Shehata Git are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and versions follow
[Semantic Versioning](https://semver.org/).

## 0.1.8 - 2026-08-01

### Changed

- The audit trail now refreshes itself every few seconds while the page is
  open, so actions performed from a terminal or a coding agent appear without
  a manual refresh. Polling stops while the window is in the background, and
  it reads only the local database — no Git or GitHub CLI process is launched.
- Overview no longer describes its state as "live", which promised realtime
  updates the app does not perform.
- Documentation leads with the problem the app solves, links directly to the
  Windows and macOS installers, and shows real product screenshots.

## 0.1.7 - 2026-08-01

### Added

- First-publish support: pushing a branch that has never been pushed now
  creates the remote branch and records it as upstream in one safe step,
  instead of failing with `no_upstream`. Smart Sync previews such branches
  as ahead-only.
- macOS CI build producing an unsigned `.dmg` artifact on every push.

### Fixed

- Pushes rejected by GitHub for a token missing the `workflow` scope now
  surface a clear, actionable message instead of a raw git error, and the
  Doctor flags signed-in accounts whose token lacks that scope.

## 0.1.6 - 2026-08-01

### Added

- Explicit, confirmed switching of the GitHub CLI default account without
  changing repository identity assignments.
- Public project documentation, sanitized screenshot placeholders, community
  templates, and clean-environment CI.
- Smart Sync readiness guidance, searchable working-tree filters, visible-file
  selection, and an in-app safe-push confirmation.

### Changed

- Renamed the ambiguous `active in gh` state to `CLI default` and clarified
  that all accounts marked `ready` remain available for repository routing.
- Refined all search controls into a consistent Liquid Glass search surface.
- Replaced remaining native Windows confirmation prompts with branded in-app
  confirmation dialogs and simplified verbatim Windows paths for display.

### Security

- Smart Sync still fetches before deciding, permits only fast-forward pulls or
  normal pushes, and stops when local and remote history diverge.
- Copyable diagnostics no longer include executable paths that can contain a
  local Windows username.

## 0.1.5 - 2026-08-01

### Added

- Searchable account and repository selectors that remain usable with long
  lists.
- Collapsible repository cards and searchable account, repository, and audit
  panels.
- Per-event audit deletion and complete local audit-history clearing.
- Real cancellation for GitHub browser login and repeat-copy confirmation for
  one-time codes.
- Official Shehata Git branding across the app, executable, Windows installer,
  and macOS icon asset.

### Changed

- Repository route checks now run concurrently with a bounded worker count.
- Query caching avoids unnecessary refreshes when navigating between panels.
- `Workspace density` is now the clearer `Layout spacing` control.

### Security

- All background Git and GitHub CLI processes remain hidden on Windows and use
  fixed executables plus argument arrays.
- Browser-login cancellation terminates the child GitHub CLI process.

## 0.1.0 - 2026-07-31

### Added

- Initial Tauri desktop, Rust core, CLI, credential helper, MCP server, SQLite
  storage, repository-scoped credential routing, safe Git actions, diagnostics,
  and NSIS packaging foundation.
