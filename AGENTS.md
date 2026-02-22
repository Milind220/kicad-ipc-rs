# AGENTS.md

Repo-scope guidance for coding agents.

## Defaults
- Keep diffs small/reviewable.
- Preserve existing patterns and naming.
- Add regression tests for bug fixes and behavior changes.
- Update docs when API/CLI behavior changes.

## Validation
- Run before handoff:
  - `cargo fmt --all`
  - `cargo test`
  - `cargo test --features blocking`

## Git Safety
- Avoid destructive git/file ops unless explicitly requested.
- Do not rewrite user changes you did not make.

## Commits
- Commit messages MUST follow Conventional Commits:
  - https://www.conventionalcommits.org/en/v1.0.0/
