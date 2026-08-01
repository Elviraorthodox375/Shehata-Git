# Security model

Shehata Git exists to make multi-account Git work *safer*, so its own security
bar is higher than average.

## What we never do

- **Never store tokens.** SQLite has no column that can hold a credential —
  there is a guard test (`schema_contains_no_secret_columns`) that fails CI if
  one is ever added.
- **Never send tokens to the frontend.** Tokens live only in the Rust backend,
  wrapped in `secrecy::SecretString`, dropped as soon as the operation ends.
- **Never log secrets.** No environment dumps, no credential-helper password
  values, no authorization headers. `shehata-core::redact` strips GitHub token
  shapes (`ghp_`, `gho_`, `ghu_`, `ghs_`, `ghr_`, `github_pat_`) from any
  free-form text before it can be logged or shown.
- **Never run shell strings.** Every external command uses argument arrays.
  Logins/hosts are validated so user input can never become a flag.
- **Never change global git config** without an explicit, explained approval.
- **Never switch the active `gh` account** as part of repository operations.

## Credential flow

```text
git push
  └─▶ git invokes git-credential-shehata get  (repo-local config)
        ├─▶ reads repo UUID from helper args
        ├─▶ opens SQLite READ-ONLY, resolves assigned account
        ├─▶ verifies host match (refuses on mismatch)
        ├─▶ gh auth token --hostname <host> --user <login>   (JIT)
        └─▶ prints username/password to git's stdout, forgets token
```

Any failure in that chain means **no credentials are emitted** — Git then
fails the operation instead of silently falling back to another account,
because the local config resets inherited helpers first.

## Backups and restore

Before Shehata Git modifies repository-local config, it records previous
values in `repository_config_backups`. **Unlink & Restore** puts them back and
removes our helper entry. The audit log notes the event.

## Reporting a vulnerability

Please do **not** open a public issue. Email the maintainer (see
`SECURITY.md` in the repository root) with a description and reproduction
steps. We will acknowledge within 72 hours.
