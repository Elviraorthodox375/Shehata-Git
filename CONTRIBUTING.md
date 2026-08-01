# Contributing

Thanks for helping build Shehata Git! This project is early — small, focused
PRs are easiest to review.

## Setup (Windows)

Prerequisites: Git, Node.js ≥ 20, pnpm ≥ 9, Rust stable (MSVC),
Microsoft C++ Build Tools, GitHub CLI, WebView2.

```bash
pnpm install
cargo build --workspace
pnpm --filter @shehata/desktop tauri dev
```

## Ground rules

1. **Business logic goes in Rust crates**, never in Tauri handlers or React.
2. **No tokens in SQLite, logs, errors, or the frontend.** The guard tests
   will catch you — that's their job.
3. **Argument arrays only** for external processes. No shell strings.
4. **No destructive Git operations.** Force push, remote deletion, hard reset,
   clean, rebase, amend are out of scope by design.
5. **No mock data in production paths.** Mocks live in tests only.
6. TypeScript strict, `cargo fmt`, `cargo clippy -D warnings` must pass.

## Commits

Conventional Commits: `feat:`, `fix:`, `docs:`, `test:`, `refactor:`,
`chore:`, optionally scoped: `feat(helper): …`.

## Pull requests

- Fill the PR template (what, why, how tested, security notes).
- Update `docs/BUILD_LOG.md` if you change behavior or architecture.
- Add/adjust an ADR in `docs/DECISIONS/` for architectural changes.

## Code of conduct

Be kind and constructive — see [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).
