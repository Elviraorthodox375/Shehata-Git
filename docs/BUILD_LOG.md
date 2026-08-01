# Shehata Git — Build Log

This log records what was actually done, verified, and found during development.
Entries are append-only and dated. Nothing is recorded here unless it really happened.

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
