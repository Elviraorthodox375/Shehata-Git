## What does this PR do?

<!-- One paragraph, plain language. -->

## Why?

<!-- Link the issue or explain the motivation. -->

## How was it tested?

- [ ] `cargo test --workspace`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `pnpm --filter @shehata/desktop typecheck`
- [ ] `pnpm --filter @shehata/desktop test`
- [ ] Manually verified in the app (describe below)

## Security notes

<!-- Required: does this touch credentials, tokens, process execution,
     git config, the database schema, or MCP tools? Explain. -->

- [ ] This PR introduces no new handling of secrets
- [ ] No new external command execution, or it uses argument arrays
- [ ] No destructive Git operations added

## Docs

- [ ] Updated docs/BUILD_LOG.md (if behavior/architecture changed)
- [ ] Added ADR in docs/DECISIONS/ (if architectural)
