# ADR 0002 — Credential routing via repository-local credential helper

- Status: Accepted
- Date: 2026-07-31

## Context

Windows users with multiple GitHub accounts get the wrong account on `git push`
because credential helpers resolve globally, not per repository.

## Decision

Each linked repository gets **local** Git config (never global):

```text
credential.helper = ""                                   (reset inherited helpers)
credential.helper = "!'<absolute-helper-path>' --repo-id <uuid>"   (our helper)
credential.useHttpPath = true
```

The fixed absolute command avoids PATH ambiguity and works for Windows install
paths containing spaces. The `!` form prevents Git for Windows from rewriting
an absolute drive path as `git-credential-<path>`. The helper reads the
repository UUID from its arguments, looks up the assigned
account in SQLite, fetches a short-lived token via
`gh auth token --hostname <host> --user <login>`, and emits it over the Git
credential protocol.

## Rules

- Before touching local config, back up existing values into
  `repository_config_backups` so **Unlink & Restore** is always possible.
- The helper fails closed: any missing mapping, account, binary, or token → no
  credentials emitted, safe diagnostics on stderr only.
- `store` is a no-op; `erase` is a no-op (lifecycle belongs to `gh`).
- A marker file `<git-dir>/shehata-git/repository-id` ties the on-disk repo to
  its database row without polluting the worktree.

## Consequences

- Works identically for terminal pushes, IDE pushes, and AI coding agents,
  because they all go through Git's standard credential resolution.
- The desktop resolves the helper beside the app (with PATH as a fallback);
  the installer still adds the install directory to **user** PATH for direct use.
- Argument-order behavior of Git credential helpers is verified by integration
  tests against the installed Git version — never assumed.
