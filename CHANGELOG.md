# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.0](https://github.com/Milind220/kicad-ipc-rs/compare/v0.4.4...v0.5.0) - 2026-05-05

### Added

- [**breaking**] update IPC bindings for KiCad 10.0.1 ([#34](https://github.com/Milind220/kicad-ipc-rs/pull/34))

### Breaking Changes

- `TitleBlockInfo.comments` now preserves fixed `comment1..comment9` slot ordering, including internal empty gaps, when round-tripping through `get_title_block_info` and `set_title_block_info`.
- Clarified `get_items_by_net` behavior for KiCad 10.0.1: net names are authoritative; net codes are legacy compatibility fields.

### Migration Guidance

- If your code assumed `TitleBlockInfo.comments` dropped all empty strings, update it to handle internal empty entries while still expecting trailing empty slots to be trimmed.
- If you deduplicated/query-filtered nets by numeric code, migrate to net-name-first logic.
- Semver note (pre-1.0): this breaking behavior belongs in the next **minor** release, not a patch release.

### Changed

- update KiCad support to 10.0.1 and align API wrappers with regenerated bindings (GetItemsByNet nets input, GetConnectedItems, SetTitleBlockInfo)
- add typed read/editable model coverage for `ReferenceImage` and `Barcode`
- refresh docs and examples for KiCad 10.0.1 command coverage and proto pin

## [0.4.4](https://github.com/Milind220/kicad-ipc-rs/compare/v0.4.3...v0.4.4) - 2026-04-25

### Added

- add editable PCB item mutation layer ([#28](https://github.com/Milind220/kicad-ipc-rs/pull/28))

## [0.4.3](https://github.com/Milind220/kicad-ipc-rs/compare/v0.4.2...v0.4.3) - 2026-03-29

### Other

- modularize client API and finalize v10 assessment follow-ups ([#25](https://github.com/Milind220/kicad-ipc-rs/pull/25))

## [0.4.2](https://github.com/Milind220/kicad-ipc-rs/compare/v0.4.1...v0.4.2) - 2026-03-29

### Added

- bump vendored KiCad protos to v10.0.0 ([#23](https://github.com/Milind220/kicad-ipc-rs/pull/23))

## [0.4.1](https://github.com/Milind220/kicad-ipc-rs/compare/v0.4.0...v0.4.1) - 2026-03-19

### Fixed

- use named pipe probe for IPC availability on Windows ([#21](https://github.com/Milind220/kicad-ipc-rs/pull/21))

## [0.4.0](https://github.com/Milind220/kicad-ipc-rs/compare/v0.3.2...v0.4.0) - 2026-03-06

### Fixed

- reduce selection API lossiness in existing public methods ([#19](https://github.com/Milind220/kicad-ipc-rs/pull/19))

## [0.3.2](https://github.com/Milind220/kicad-ipc-rs/compare/v0.3.1...v0.3.2) - 2026-03-02

### Fixed

- decouple project commands from GetOpenDocuments and add KIPRJMOD fallback ([#16](https://github.com/Milind220/kicad-ipc-rs/pull/16))

### Other

- add mdBook guide site and Pages deploy workflow ([#15](https://github.com/Milind220/kicad-ipc-rs/pull/15))

## [0.3.1](https://github.com/Milind220/kicad-ipc-rs/compare/v0.3.0...v0.3.1) - 2026-02-28

### Fixed

- *(api)* strengthen crate-level and high-impact API docs ([#13](https://github.com/Milind220/kicad-ipc-rs/pull/13))

## [0.3.0](https://github.com/Milind220/kicad-ipc-rs/compare/v0.2.0...v0.3.0) - 2026-02-22

### Added

- expose via layer spans in typed model and CLI
