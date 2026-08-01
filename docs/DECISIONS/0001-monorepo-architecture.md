# ADR 0001 — Monorepo architecture and technology stack

- Status: Accepted
- Date: 2026-07-31

## Context

Shehata Git is a Windows-first desktop app with a CLI, a Git credential helper,
and an MCP server. All of them must share the same business logic so behavior is
identical whether a push comes from the UI, a terminal, or an AI coding agent.

## Decision

Use a single monorepo with:

- **Tauri 2 + React + TypeScript + Vite + Tailwind CSS + shadcn/ui** for the desktop app (`apps/desktop`)
- **Rust workspace** under `crates/` holding all business logic:
  - `shehata-core` — orchestration, domain models, shared error model
  - `shehata-storage` — SQLite (rusqlite) with embedded migrations
  - `shehata-git` — system `git` executable wrapper (no libgit2)
  - `shehata-github` — `gh` CLI wrapper + token provider (secrecy/zeroize)
  - `shehata-cli` — `shehata` binary
  - `shehata-credential-helper` — `git-credential-shehata` binary
  - `shehata-mcp` — `shehata-mcp` stdio MCP server binary
- **pnpm workspace** for the desktop frontend, leaving room for shared packages
  only when a concrete reuse boundary appears

Dependency direction: UI/CLI/MCP → `shehata-core` → storage/git/github.
Business logic never lives in Tauri command handlers or React components.

## Why system `git` and `gh` executables

- Behavior must be byte-identical to what terminals and AI agents experience.
- `gh` is the credential source of truth; we never implement a custom OAuth app in v0.1.
- Tokens are fetched just-in-time via `gh auth token --user <login>` and held only in memory.

## Consequences

- Requires Rust + MSVC Build Tools on Windows dev machines.
- App-launched external commands use fixed executables and argument arrays. The
  sole shell-form exception is Git's required repository-local `!` credential
  helper entry, which is built from a canonical executable path and validated
  repository UUID.
- Every crate is independently testable with fake `git`/`gh` binaries injected via PATH.
