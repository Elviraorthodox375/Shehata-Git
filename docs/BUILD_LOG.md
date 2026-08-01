# Shehata Git — Build Log

This log records what was actually done, verified, and found during development.
Entries are append-only and dated. Nothing is recorded here unless it really happened.

---

## 2026-08-01 — Phase 10 Windows installer vertical slice

- Added a reproducible sidecar preparation step that release-builds the CLI,
  credential helper, and MCP server, names them with the native Rust target
  triple, and places them where Tauri verifies them for bundling.
- Configured the NSIS bundle to ship `shehata-git.exe`, `shehata.exe`,
  `git-credential-shehata.exe`, and `shehata-mcp.exe` together.
- Added installer-only current-user PATH maintenance through the native CLI.
  It preserves the existing registry value type and contents, rejects invalid
  paths, avoids duplicates case-insensitively, and broadcasts the Windows
  environment-change notification.
- Added NSIS post-install and pre-uninstall hooks so terminals and Git can find
  the packaged commands without administrator privileges.
- Added unit coverage for idempotent PATH insertion and exact removal while
  preserving unrelated entries.
- Built the real unsigned Windows x64 NSIS installer at
  `target/release/bundle/nsis/Shehata Git_0.1.0_x64-setup.exe`.
- Inspected the generated NSIS script and confirmed that all three external
  binaries are copied on install and removed on uninstall.
- Ran a real silent smoke test into a workspace-bounded temporary directory:
  all four executables and the uninstaller were present, the installed CLI
  reported `shehata 0.1.0`, the directory appeared in user PATH, uninstall
  removed it, the exact original PATH was restored, and the install directory
  was removed.
- The installer is intentionally unsigned. Code signing, hosted draft-release
  execution, accessibility review, and owner two-account acceptance remain.
- Final repository gate passed: workspace formatting, clippy with warnings
  denied, all 69 Rust tests, strict TypeScript, 3 frontend tests, and Biome lint.

---

## 2026-08-01 — Phase 9 safe native MCP surface

- Replaced every MCP milestone placeholder with shared-core implementations for status, diff counts, identity, connection test, explicit-path commit, pull `--ff-only`, and normal push.
- MCP commit stages only explicit repository-relative paths. It never exposes arbitrary shell, force push, remote deletion, amend, rebase, hard reset, or clean.
- MCP push always identifies its caller as AI and always passes `approved=false`; `ask_before_push` returns `approval_required`, while `block_ai_push` returns `operation_blocked`.
- Repository arguments use strict schemas and require exactly one UUID or path. Domain failures return the stable `{ ok, code, summary, data }` envelope.
- Diff summary returns counts only and never returns file contents. Tokens remain inside the credential helper/GitHub CLI path and never enter tool results.
- Added a process-level MCP protocol contract test that initializes the real stdio server, lists exactly the reviewed 11-tool surface, rejects force/shell/reset/delete tool exposure, invokes a tool, and validates its structured credential-free error envelope.
- Added recoverable bounded AGENTS.md generation that preserves existing project instructions, updates only the marked Shehata Git block, rejects malformed/duplicate markers, and keeps a temporary rollback file until replacement succeeds.
- Added the desktop repository selector and Generate AGENTS.md control; the existing page continues to show the exact MCP executable and copyable client configuration.
- External MCP Inspector/client acceptance remains pending the packaged binaries.
- Final repository gate passed: workspace formatting, clippy with warnings denied, all 67 Rust tests, strict TypeScript, 3 frontend tests, and Biome lint.

---

## 2026-08-01 — Phase 8 native CLI commands

- Replaced CLI milestone placeholders with commands that call the same shared Rust core as the desktop.
- Implemented accounts list/refresh; repositories list/add/show/assign-and-route/unlink-and-restore; status; non-mutating connection test; normal push with preflight; and fixed native MCP launcher.
- Repository references accept a stable UUID, repository root, any nested path inside a registered worktree, or the current directory where documented.
- Added human-readable output and global `--json` output with stable error codes. Human output escapes terminal control characters; neither mode outputs tokens.
- Added `push --yes` for explicit approval-policy acknowledgement. No force, force-with-lease, arbitrary command, amend, rebase, destructive reset, or remote-deletion option exists.
- The CLI locates `shehata-mcp` only beside itself or on PATH and launches that fixed executable with inherited stdio.
- Added CLI schema and process-level contract tests covering command shape, absence of force options, structured JSON errors, and credential-output guards.
- Built and manually verified `shehata --help`, `shehata repos --help`, and `shehata --json repos list`.
- Installer/PATH delivery remains in Phase 10.
- Final repository gate passed: workspace formatting, clippy with warnings denied, all 64 Rust tests, strict TypeScript, 3 frontend tests, and Biome lint.

---

## 2026-08-01 — Phase 7 safe local Git actions foundation

- Added shared status, selected-path stage/unstage, and normal commit services in Rust core.
- Dynamic paths are bounded, must remain repository-relative, cannot target `.git`, and are always passed after Git's `--` option terminator without a shell.
- Unstage works safely both with an existing HEAD (`git restore --staged`) and on an unborn branch (`git rm --cached`), without changing worktree files.
- Commit rejects empty staged sets and unresolved conflicts, never exposes amend, and records safe success/failure audit events.
- Added a real temporary-repository test covering status, first-commit stage/unstage, normal commit, and clean post-commit status.
- Added a professional Changes dialog to repository rows with file-state selection, Stage selected, Unstage selected, and commit-message controls.
- Added pull using fixed `--ff-only` arguments and normal push using an explicit existing upstream destination; force, force-with-lease, deletion, amend, rebase, hard reset, and arbitrary commands are not exposed.
- Push preflight verifies the exact installed helper configuration, assigned account token availability, HTTPS host/owner/repository match, branch/ref safety, attached HEAD, conflicts, upstream, freshly fetched ahead/behind state, non-fast-forward risk, and repository policy.
- Added safe audit records for preflight, pull, push, policy changes, successes, failures, and blocks without storing command output or credentials.
- Added repository policies for allow normal push, require approval, and block AI push, with a validated selector in the Changes dialog.
- Added a real two-worktree/local-bare-remote test proving fast-forward-only pull and normal push behavior with the fixed production command builders.
- Rejected unencrypted HTTP remote URLs; automatic credential routing remains HTTPS-only.
- Built and visually inspected the native debug app and Repositories page. The Changes dialog was not opened because the owner's app database has no authenticated account/repository, and visual QA did not create fake persistent user data.
- Real GitHub pull/push acceptance through the owner's two authenticated accounts remains pending.
- Final repository gate passed: workspace formatting, clippy with warnings denied, all 60 Rust tests, strict TypeScript, 3 frontend tests, and Biome lint.

---

## 2026-08-01 — Phase 6 credential routing vertical slice

### Repository-scoped credential routing

- Added the shared Phase 6 routing service for enable, connection test, and unlink/restore operations.
- Routing writes only repository-local Git configuration: an empty helper reset, the fixed Shehata helper command with repository UUID, and `credential.useHttpPath=true`.
- Preserves empty and multi-valued Git configuration exactly, backs up original values before mutation, verifies applied values, and restores them on failure or unlink.
- Uses the absolute helper path safely on Windows, including install paths containing spaces and extended `\\?\` paths.
- Connection testing runs non-interactive `git ls-remote <remote> HEAD` and records only safe audit metadata.
- Unlink restores credential configuration, removes only a matching repository marker, clears the account mapping, and optionally restores or preserves local commit identity.

### End-to-end helper proof and desktop controls

- Added a real Git integration test that configures the built helper, runs `git credential fill`, resolves the repository assignment from SQLite, calls a fake `gh` executable with the exact host/login arguments, and receives the expected credential output.
- The integration test covers a repository and application path containing spaces and caught/fixed a Windows helper-command rewriting bug.
- Repository rows now show actual routing state read from local Git config and expose Enable route, Verify, and confirmed Unlink controls.
- No real GitHub token was used, persisted, logged, or sent to the frontend. Real owner `ls-remote` and external-push acceptance remain pending authenticated accounts.
- Final repository gate passed: workspace formatting, clippy with warnings denied, all 53 Rust tests, strict TypeScript, 3 frontend tests, and Biome lint.

---

## 2026-08-01 — Phase 5 assignment foundation and professional UI refresh

### Repository assignment and local identity

- Added a shared Phase 5 assignment service used by the desktop bridge.
- Assignment resolves an exact `host + login` from the safe account mirror and refuses unavailable accounts or host mismatches.
- Creates `<git-dir>/shehata-git/repository-id` without overwriting a conflicting marker.
- Supports optional repository-local `user.name` and `user.email` changes only; input is trimmed, bounded, and rejects control characters or malformed email addresses.
- Saves the original local identity values before the first change and preserves those backups across later reassignments.
- Rolls back identity changes and a newly created marker when a later assignment step fails.
- Added a real temporary-Git-repository test proving account assignment, marker creation, local identity changes, database persistence, and backup contents.

### Desktop assignment flow

- Added the Tauri assignment command and typed frontend bridge.
- Repository rows now expose Assign identity / Edit assignment.
- Added a confirmation dialog that filters accounts by token availability and remote host, shows SSH routing limitations, and makes local-only identity behavior explicit.
- Successful assignment refreshes the repository route in the UI without exposing credentials.

### Visual redesign

- Reworked the application into a precision desktop-tool aesthetic instead of a generic card dashboard.
- Added a structured workspace sidebar, local/security status header, technical typography, subtle grid canvas, compact status rails, and shared instrument-panel styling.
- Redesigned onboarding, Overview, Identities, and Repositories with consistent registry language and responsive layouts.
- Kept motion restrained and respects the existing reduced-motion rule.
- Visually verified the rebuilt native app across Overview, Repositories, and Identities. No browser authentication or user-repository assignment was triggered during visual QA.
- Native debug build completed with `--no-bundle`; no installer was generated.
- Final repository gate passed: workspace formatting, clippy with warnings denied, all 51 Rust tests, strict TypeScript, 3 frontend tests, and Biome lint.

---

## 2026-08-01 — Phase 4 repository discovery and native folder picker

### Repository discovery

- Added read-only repository discovery to `shehata-git` using only argument-array Git commands.
- Canonicalizes the selected folder and validates that it is a Git worktree.
- Reads the top-level worktree path, Git directory, common Git directory, branch or detached state, HEAD, upstream, remotes, ahead/behind counts, working-tree summary, local commit identity, and existing local credential settings.
- Parses HTTPS and SSH GitHub remotes without changing them. Unsupported/local remotes remain visible without being misidentified as GitHub.
- Added temporary-repository tests for valid worktrees, non-repository folders, status parsing, and primary-remote selection.

### Persistence and desktop flow

- Added shared core orchestration that persists discovered metadata in SQLite without holding a database connection across an await point.
- Re-adding a known canonical path refreshes discovery metadata while preserving its stable id, assigned account, push policy, and creation time.
- Added a Tauri command and native Windows folder picker capability for selecting one repository folder.
- Replaced the disabled placeholder with a functional Add repository flow and repository cards showing branch, remote protocol, GitHub path, and assignment state.
- No repository Git configuration, remote, credentials, or global settings are changed during discovery.

### Verification

- Targeted Rust tests passed for `shehata-git`, `shehata-storage`, `shehata-core`, and the desktop bridge.
- Final repository gate passed: workspace formatting, clippy with warnings denied, all 49 Rust tests, strict TypeScript, 3 frontend tests, and Biome lint.
- Frontend strict typecheck, tests, Biome lint, and production Vite build passed.
- Native Tauri debug application built successfully with `--no-bundle`; no installer was generated.
- Visually verified the Repositories page and confirmed Add repository opens the native Windows folder picker. The picker was cancelled, so no user repository was saved during UI verification.

---

## 2026-08-01 — Recovery audit, Phase 1/2 verification, and Phase 3 browser login

### Recovered state

- Read the original 37 KB build prompt and audited the repository left by the previous agent.
- Confirmed the repository still has no commits and all project files are untracked.
- Confirmed there was no `HANDOFF.md`; the previous build log only described Phase 0 even though later-phase code existed.
- The Shehata Git MCP repository tools requested by `AGENTS.md` were not available in the Codex session. Git inspection was read-only; no commit, push, remote, credential, or global Git configuration change was made.

### Baseline fixes

- Migrated `biome.json` to the installed Biome 2.5.6 schema, restored the intended source-file scope, enabled Tailwind directive parsing, and fixed frontend accessibility lint errors.
- Applied Rust formatting and fixed all workspace clippy/build failures found in storage, core error serialization, Tauri configuration, account discovery, and the early MCP server.
- Moved NSIS settings to `bundle.windows.nsis`, matching the current Tauri 2 configuration schema.
- Refactored async GitHub account discovery so a non-`Sync` rusqlite connection is never held across an await point.
- Enabled rmcp's `transport-io` feature and made MCP structured envelopes compatible with rmcp 3.1 output schemas.
- Made writable `Database::open_at` create missing parent directories; added a regression test. The first real Doctor run had exposed this first-launch failure.
- Removed a Doctor repair hint that referenced an unimplemented `--fix-path` command.

### Phase 1 and Phase 2 verification

- Built the production frontend and a native Tauri debug application.
- Launched `target/debug/shehata-git.exe` and visually verified the branded Welcome screen.
- Navigated to the live System Check screen and verified that it rendered real machine results.
- Current real Doctor results: Git, GitHub CLI, SQLite, credential helper, WebView2, and MCP server are ready. GitHub accounts and the development binary PATH require attention, as expected.
- Found and documented a build-order pitfall: a later plain `cargo build --workspace` can overwrite the Tauri-produced debug executable with a development binary that expects `localhost:1420`. A Tauri build must be the final application build before visual/manual testing.

### Phase 3 browser login

- Added a safe GitHub CLI browser-login runner using the fixed command: `gh auth login --hostname github.com --git-protocol https --web --clipboard`.
- Added Tauri Channel progress events for started, waiting-for-browser, and validated one-time device-code states.
- Raw stdout/stderr is never sent to the frontend; tokens are never requested by the login flow.
- Added a responsive, keyboard-accessible Add GitHub Account experience with progress, one-time-code, success, and failure states.
- Added a Windows fake-`gh` regression test proving that only curated events leave the runner and raw diagnostics do not.
- Real account authentication was intentionally not performed; it requires the owner to complete GitHub's browser flow.

### Verified checks during recovery

- `cargo clippy --workspace --all-targets -- -D warnings` passed after fixes.
- `cargo test --workspace` passed with 40 Rust tests before the Phase 3 additions.
- `cargo test -p shehata-storage` passed with 10 tests after the missing-parent regression test.
- `cargo test -p shehata-github` passed with 7 tests after the browser-login fake-`gh` test.
- Frontend lint, strict typecheck, tests (3), and production Vite build passed.
- Native Tauri debug build completed and produced `target/debug/shehata-git.exe`.
- Final post-Phase-3 quality gate passed: formatting, workspace clippy with warnings denied, 43 Rust tests, strict TypeScript, 3 frontend tests, and Biome lint.
- Rebuilt the Tauri application after the final Cargo command and visually verified the new Accounts page and both Add GitHub Account entry points. Real browser authentication was not started.

---

## 2026-07-31 — Phase 0: Environment inspection

### Machine

- OS: Windows 11 (build 26200), x64
- Shell used for build: Git Bash
- Working directory: `D:\Pormpt Marketing Agency\Shehata Git` (was empty, now the repo root)

### Toolchain status

| Tool | Status | Version / Action |
|---|---|---|
| Git | ✅ present | 2.55.0.windows.3 |
| Node.js | ✅ present | v24.12.0 |
| pnpm | ✅ present | 9.15.9 |
| WebView2 Runtime | ✅ present | 150.0.4078.105 |
| GitHub CLI (`gh`) | ✅ installed during Phase 0 | 2.97.0 via `winget install GitHub.cli` (owner approved) |
| Rust (rustup) | ✅ installed during Phase 0 | rustup 1.29.0, cargo/rustc 1.97.1 via `winget install Rustlang.Rustup` (owner approved) |
| MSVC C++ Build Tools | ⏳ installing in background | VS 2022 Build Tools + VCTools workload + Windows 11 SDK 22621 via winget (owner approved). Required for linking Rust MSVC binaries and Tauri builds |

### Notes

- `gh auth status` not yet run — no GitHub accounts configured on this machine yet. Owner will authenticate in Phase 3 via browser login from inside the app.
- No global Git configuration has been modified.
- Nothing has been pushed to any remote.

---
