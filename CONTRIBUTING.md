# Contributing

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

## Maintainer Notes
- Proto regeneration workflow lives in `CONTRIBUTIONS.md`.
