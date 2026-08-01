# Roadmap

This roadmap describes direction, not guaranteed dates. Security and data
integrity take priority over feature count.

Status: ✅ complete · 🚧 in progress · ⬜ planned

## Foundation ✅

- Tauri 2 + React desktop application with a shared Rust business layer.
- SQLite persistence with an enforced no-credential schema.
- GitHub CLI account discovery and browser sign-in.
- Canonical repository discovery and repository-local identity assignment.
- HTTPS credential routing through `git-credential-shehata`.
- Safe status, diff, stage, unstage, commit, fast-forward pull, and normal push.
- CLI and bounded MCP surfaces using the same core policies.
- Redacted local activity history, diagnostics, and exact unlink/restore.
- Windows NSIS packaging with CLI/helper/MCP sidecars and user PATH setup.

## Public Windows beta 🚧

- ✅ Scalable searchable selectors and collapsible repository cards.
- ✅ Explicit GitHub CLI default-account switching.
- ✅ Public documentation, contribution policy, security policy, issue/PR
  templates, and continuous integration.
- ⬜ Complete the two-account manual acceptance checklist on disposable private
  repositories.
- ⬜ Capture sanitized screenshots and short onboarding media.
- ⬜ Sign the Windows executable and installer.
- ⬜ Add signed automatic updates with an explicit user-controlled channel.
- ⬜ Publish the first reviewed prerelease after the checklist passes.

## Cross-platform preview ⬜

- Build and test on Apple silicon and Intel macOS.
- Add macOS signing, hardened runtime, and notarized DMG distribution.
- Validate Linux WebKit packaging and desktop integration.
- Replace Windows-only prerequisite installation with platform-specific,
  reviewable guidance.
- Verify keychain and credential-helper behavior on every supported platform.

## Workflow expansion ⬜

- First-class SSH routing with the same fail-closed identity guarantees.
- Optional repository groups and bulk read-only health checks.
- Exportable redacted diagnostics and audit records.
- Expanded accessibility and localization coverage.
- More Git hosts only after the GitHub security model is mature.

## Possible team edition — validation required ⬜

- Optional organization policy and device inventory.
- Shared policy metadata and compliance reporting without uploading source code,
  repository contents, or credentials.
- License/entitlement service separated from the local routing core.

This section will not be built until paid pilot users validate the need.

## Explicit non-goals

- Storing, syncing, or sharing GitHub tokens
- Arbitrary shell execution through MCP
- Force push, destructive reset, clean, rebase, amend, or remote deletion
- Replacing Git, GitHub CLI, or a full Git hosting client
- Uploading repository source code to a Shehata Git service
