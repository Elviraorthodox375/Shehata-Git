# Changelog

All notable changes to Shehata Git are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
versioning follows [SemVer](https://semver.org/).

## [Unreleased]

### Added

- Monorepo scaffold: Tauri 2 + React 19 + strict TypeScript desktop app,
  Cargo workspace with 7 crates, pnpm workspace.
- Brand system: original S-node SVG mark, dark/light design tokens,
  generated Windows app icons.
- Full app shell: Home, Accounts, Repositories, AI Integration, Activity,
  Settings, first-run onboarding flow.
- System doctor: 8 live checks (git, GitHub CLI, accounts, SQLite, credential
  helper, WebView2, PATH, MCP server) with plain-language repair hints,
  in-app page and `shehata doctor` CLI (`--json`, exit code 4 when unhealthy).
- Shared error model with stable machine codes across UI/CLI/MCP.
- `git-credential-shehata`: credential-helper protocol implementation
  (get/store/erase), fail-closed resolution of repository → account →
  just-in-time token via GitHub CLI; read-only DB access.
- `shehata-mcp`: stdio MCP server (official rmcp SDK) exposing
  doctor / accounts / repositories / identity tools with structured
  envelopes; remaining v0.1 tools return honest `not_implemented` codes.
- SQLite storage with embedded migrations and a guard test proving no
  column can ever hold a credential.
- Secret redaction utilities with unit tests for all GitHub token shapes.
