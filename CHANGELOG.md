# Changelog

All notable changes to Shehata Git are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and versions follow
[Semantic Versioning](https://semver.org/).

## 0.1.15 - 2026-08-02

### Fixed

- A network action no longer refuses to run because of a stale account state.
  A token probe that failed during an outage used to stay recorded as
  unavailable until the accounts page was refreshed by hand; live GitHub CLI
  state is now re-read once before the action is refused. Routing still fails
  closed — it never falls through to a different account.

## 0.1.14 - 2026-08-02

### Changed

- Failed and blocked network actions now carry the same context as successful
  ones — repository, branch, and remote — instead of one bare sentence. A
  failure is when that context matters most.
- The activity trail can be sorted newest or oldest first.

## 0.1.13 - 2026-08-02

### Changed

- Activity entries are now two lines instead of one long sentence: the change
  itself is the title, and repository, branch, and short commit sit on a
  quieter line beneath it. Pushes stay labelled "Normal push" there, because
  the trail is where the never-force-push guarantee has to remain visible.
- Activity search also matches the repository, branch, and commit line.

## 0.1.12 - 2026-08-02

### Changed

- Activity entries now identify what an action touched: repository, branch,
  short commit, and the commit subject — instead of one fixed sentence that
  looked identical for every repository. Commit subjects are redacted and
  truncated before being stored.

## 0.1.11 - 2026-08-01

### Added

- `shehata gh <command>` runs any GitHub CLI command as the account assigned to
  the current repository, then restores the previous CLI default. Git already
  routed per repository; this closes the same gap for `gh` commands such as
  `gh pr create`. The passthrough is command-line only and is not exposed to
  the desktop app or the MCP server.

## 0.1.10 - 2026-08-01

### Changed

- Automatic setup now detects whether Windows Package Manager exists before
  offering to install Git and GitHub CLI. When it is missing, the panel
  explains why and links to App Installer instead of failing after the click.

## 0.1.9 - 2026-08-01

### Added

- System check can now repair a missing `workflow` scope in place. Choosing
  **Grant workflow access** opens GitHub's own approval flow for that exact
  account and restores the previous CLI default account afterwards, including
  when the request fails or is cancelled.

### Security

- Only scopes on an explicit allow-list may be requested from the GitHub CLI,
  so a future caller cannot widen an account's permissions by accident.

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
