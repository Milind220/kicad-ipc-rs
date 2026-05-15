# KiCad 11 IPC Plan

Read when: working on KiCad 11 / 10.99 IPC support.

## Snapshot

- KiCad submodule: `d3bb3db575303c86edd68d526258d6e5b9294bc9`
- Proto version string: `10.99.0-991-gd3bb3db575`
- Source ref used: `origin/master`
- Branch status: unstable groundwork; not stable KiCad 11 support

## Implemented Groundwork

- Generated KiCad 11 protos compile under `src/proto/generated`.
- New generated modules included:
  - `kiapi.board.jobs`
  - `kiapi.schematic.jobs`
- Generated protos are public under `kicad_ipc_rs::proto::kiapi`.
- `KiCadClient::send_raw_command` sends pre-packed `prost_types::Any` commands.
- `commands::*` modules expose type constants and raw packers for new command families.
- Existing KiCad 10 wrappers are kept where the KiCad 11 schema still compiles.

## Command Family Status

| Family | KiCad commands | Current crate status | Typed wrapper status |
| --- | --- | --- | --- |
| Common project | `OpenDocument`, `CloseDocument`, `SaveDocument` | raw helpers in `commands::project` | pending |
| Common editor | `GetPageSettings`, `SetPageSettings` | raw helpers in `commands::editor` | pending |
| Board design rules | `GetBoardDesignRules`, `SetBoardDesignRules`, `GetCustomDesignRules`, `SetCustomDesignRules` | raw helpers in `commands::board` | pending |
| Board jobs | `RunBoardJobExport3D`, `RunBoardJobExportRender`, `RunBoardJobExportSvg`, `RunBoardJobExportDxf`, `RunBoardJobExportPdf`, `RunBoardJobExportPs`, `RunBoardJobExportGerbers`, `RunBoardJobExportDrill`, `RunBoardJobExportPosition`, `RunBoardJobExportGencad`, `RunBoardJobExportIpc2581`, `RunBoardJobExportIpcD356`, `RunBoardJobExportODB`, `RunBoardJobExportStats` | raw helpers in `commands::board` | pending |
| Schematic model | `GetSchematicHierarchy`, `GetSchematicNetlist` | raw helpers in `commands::schematic` | pending |
| Schematic jobs | `RunSchematicJobExportSvg`, `RunSchematicJobExportDxf`, `RunSchematicJobExportPdf`, `RunSchematicJobExportPs`, `RunSchematicJobExportNetlist`, `RunSchematicJobExportBOM` | raw helpers in `commands::schematic` | pending |

## Compatibility Notes

- `BeginCommit` and `EndCommit` now include an item header for the active board document.
- `TextAttributes.color` is not represented in `TextAttributesSpec` yet; outgoing typed text uses `None`.
- New graphic ellipse and ellipse-arc variants are recognized in item kind strings, but the existing typed geometry models do not expose full ellipse attributes yet.
- Board/schematic job structs are exposed raw-first through generated prost types.

## Next Typed Wrapper Work

- Add typed document lifecycle APIs that support board and schematic documents, not only the active board.
- Add page settings model and get/set wrappers.
- Add board design rule models and custom design rule update flow.
- Add schematic hierarchy and netlist read models.
- Add job settings builders and response decoding for board and schematic export jobs.
- Decide whether ellipse and ellipse-arc geometry should be first-class public model variants.

## Validation

Run before handoff on this branch:

```bash
cargo fmt --all
cargo check
cargo test
cargo test --features blocking
```
