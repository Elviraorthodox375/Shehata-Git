# Testing

## Automated

```bash
# Rust: unit + integration (uses temp dirs and fake binaries, never real GitHub)
cargo test --workspace

# Frontend
pnpm --filter @shehata/desktop test

# Lint / types / format
pnpm --filter @shehata/desktop typecheck
pnpm --filter @shehata/desktop lint
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```

### Rules for tests

- No real GitHub calls in automated tests — fake `gh` binaries earlier in PATH.
- Temporary directories for every repository/database a test touches.
- Guard tests assert that no token can appear in the schema, logs, or errors.

## Manual two-account acceptance checklist

> Run this only with **two disposable private test repositories** and
> **two GitHub accounts you own**. Never on important repositories.

Setup:

- [ ] Account A and account B both signed in (Accounts page shows both)
- [ ] Two test repos cloned locally, e.g. `test-a` (owned by A), `test-b` (owned by B)

Routing:

- [ ] Link `test-a`, assign account A, set local name/email, test connection → success
- [ ] Link `test-b`, assign account B, set local name/email, test connection → success
- [ ] In `test-a`: commit a change and push from the **app** → GitHub shows the commit **by account A**
- [ ] In `test-b`: commit a change and push from **PowerShell** (`git push`) → commit **by account B**
- [ ] Push `test-a` from an **AI coding tool** (via MCP or its terminal) → commit **by account A**
- [ ] Windows Credential Manager was never opened during any of the above

Safety:

- [ ] `git push --force` is not offered anywhere in app/CLI/MCP
- [ ] Unlink `test-a` → previous credential config restored, repo untouched
- [ ] Database contains no tokens (inspect with any SQLite browser)
- [ ] Activity page shows the events with no sensitive data

## Verifying a release build

```bash
pnpm build
# Installer: target/release/bundle/nsis/Shehata Git_<version>_x64-setup.exe
```

The Windows smoke test must verify all of the following with a temporary install:

- `shehata-git.exe`, `shehata.exe`, `git-credential-shehata.exe`, and
  `shehata-mcp.exe` are installed beside one another.
- `shehata --version` runs from the installed copy.
- The install directory is added exactly once to the current-user PATH.
- Silent uninstall succeeds, removes the install directory, and restores the
  exact original current-user PATH value.
- The unsigned development installer is never published as a stable release;
  a human must review the draft and the expected SmartScreen warning first.
