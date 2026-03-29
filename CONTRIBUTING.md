# Contributing

Issues and PRs welcome! This document covers the contribution workflow.

This repository requires Conventional Commits.

## Commit Message Policy (Required)
- All commits MUST follow the Conventional Commits 1.0.0 spec:
  - https://www.conventionalcommits.org/en/v1.0.0/
- Allowed types in this repo:
  - `feat`, `fix`, `refactor`, `build`, `ci`, `chore`, `docs`, `style`, `perf`, `test`

Examples:
- `feat(client): expose via layer span in typed model`
- `fix(cli): parse board-origin --type drill correctly`
- `test(client): cover via padstack layer decoding`

## Before Opening a PR
- Run:
  - `cargo fmt --all`
  - `cargo test`
  - `cargo test --features blocking`

## Resources
- Guide site source: `docs/book/src/` (deployed via GitHub Pages)
- Proto regeneration workflow: `CONTRIBUTIONS.md`
