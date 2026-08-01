# Contributing to Shehata Git

Thanks for helping improve **Shehata Git — One repo. One identity. Zero
switching.** Focused issues and small pull requests are the easiest to review.

## Before you start

- Search existing issues and pull requests.
- Open an issue before a large feature or architectural change.
- Never include credentials, real access tokens, private repository URLs,
  customer data, or machine-specific paths in an issue, fixture, screenshot, or
  commit.
- Security vulnerabilities belong in a private GitHub Security Advisory; see
  [SECURITY.md](SECURITY.md).

## Development setup

Install Node.js 20+, pnpm 9+, stable Rust, Git, GitHub CLI, and the
[Tauri platform prerequisites](https://v2.tauri.app/start/prerequisites/).

```bash
git clone https://github.com/moshehata95/Shehata-Git.git
cd Shehata-Git
pnpm install --frozen-lockfile
pnpm prepare:sidecars
cargo build --workspace
pnpm dev
```

Run `pnpm prepare:sidecars` once after a clean clone and again after changing a
sidecar binary. Tauri requires those generated executables while compiling the
desktop crate; they remain ignored build artifacts.

Automated tests use temporary repositories and fake GitHub CLI binaries. Do not
use a real token or private repository to create a test fixture.

## Architecture rules

1. Business logic belongs in `crates/shehata-core`, not React components or
   Tauri command handlers.
2. Tokens must never enter SQLite, logs, frontend state, diagnostics, or MCP
   responses.
3. Programs launched by Shehata Git must use an executable plus argument array,
   validated inputs, and a timeout. The Git-required `!` credential-helper
   entry is the sole exception and must remain strictly constructed from a
   canonical executable path and validated UUID.
4. Destructive Git operations remain out of scope: no force push, hard reset,
   clean, rebase, amend, or remote deletion.
5. Repository configuration changes require backup, verification, and an exact
   restore path.
6. New architectural decisions require an ADR in `docs/DECISIONS/`.

## Required checks

Run this complete gate before committing:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm --filter @shehata/desktop lint
pnpm --filter @shehata/desktop typecheck
pnpm --filter @shehata/desktop test
pnpm --filter @shehata/desktop build
```

For UI changes, also test keyboard navigation, 200% zoom, reduced motion,
reduced transparency, and both layout-spacing modes.

### Disk space

Repeated debug builds of this workspace grow past 15 GB. Reclaim that space
without touching release artifacts:

```bash
cargo clean --profile dev
```

## Commits and pull requests

Use Conventional Commits, for example:

```text
feat(accounts): add explicit CLI default switch
fix(routing): preserve helper backup during relink
docs: clarify clean source build
```

Complete the pull request template, describe security-sensitive behavior, list
the exact checks run, and include only sanitized screenshots. Update
`CHANGELOG.md` for user-visible changes and `docs/BUILD_LOG.md` for verified
implementation work.

By contributing, you agree that your contribution is licensed under the
[MIT License](LICENSE).
