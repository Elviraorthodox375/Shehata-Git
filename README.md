# Shehata Git

**One repo. One identity. Zero switching.**

Shehata Git is a local-first desktop identity manager that maps each Git repository to the correct GitHub account—so developers and AI coding agents can push without manually switching credentials.

> ⚠️ **Early development warning**
> Shehata Git is in active early development (v0.1). Expect rough edges, and do
> not use it on important repositories yet. See the
> [manual acceptance checklist](docs/TESTING.md) before relying on it.

## What it does

- Shows all GitHub accounts authenticated through the official GitHub CLI
- Lets you assign **one account per local repository**
- Routes every `git push` — from the app, PowerShell, Cursor, Claude Code,
  Codex, Kimi CLI, or OpenCode — through the correct account automatically
- Fails safely instead of silently using the wrong account
- Never stores tokens: credentials stay in the GitHub CLI, fetched just-in-time

## What it is not

Shehata Git is **not** a replacement for GitHub Desktop or Git itself. It is a
visual Git identity manager for humans and AI coding agents.

## Documentation

- [Architecture](docs/ARCHITECTURE.md)
- [Security model](docs/SECURITY.md)
- [Roadmap](docs/ROADMAP.md)
- [Troubleshooting](docs/TROUBLESHOOTING.md)
- [Testing](docs/TESTING.md)
- [Build log](docs/BUILD_LOG.md)
- [Decision records](docs/DECISIONS/)

## Development

Prerequisites (Windows): Git, Node.js LTS, pnpm, Rust (MSVC), Microsoft C++
Build Tools, GitHub CLI, WebView2 Runtime.

```bash
pnpm install
cargo build
pnpm --filter @shehata/desktop tauri dev
```

## License

[MIT](LICENSE)
