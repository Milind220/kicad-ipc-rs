# Selection API Lossiness Audit + Execution Plan

Goal: close data-loss gaps between KiCad protobuf payloads and public `kicad-ipc-rs` selection APIs.

## Scope

- `GetSelection` family:
  - `get_selection_raw`
  - `get_selection`
  - `get_selection_details`
  - `get_selection_summary`
  - `add/remove/clear_selection` typed wrappers
  - `get_selection_as_string`

## Source Anchors (do not re-discover)

- Proto commands:
  - [`kicad/api/proto/common/commands/editor_commands.proto:338`](/Users/milindsharma/Developer/kicad-oss/kicad-ipc-rs/kicad/api/proto/common/commands/editor_commands.proto:338) (`GetSelection`)
  - [`kicad/api/proto/common/commands/editor_commands.proto:349`](/Users/milindsharma/Developer/kicad-oss/kicad-ipc-rs/kicad/api/proto/common/commands/editor_commands.proto:349) (`SelectionResponse`)
  - [`kicad/api/proto/common/commands/editor_commands.proto:355`](/Users/milindsharma/Developer/kicad-oss/kicad-ipc-rs/kicad/api/proto/common/commands/editor_commands.proto:355) (`AddToSelection`)
  - [`kicad/api/proto/common/commands/editor_commands.proto:364`](/Users/milindsharma/Developer/kicad-oss/kicad-ipc-rs/kicad/api/proto/common/commands/editor_commands.proto:364) (`RemoveFromSelection`)
  - [`kicad/api/proto/common/commands/editor_commands.proto:373`](/Users/milindsharma/Developer/kicad-oss/kicad-ipc-rs/kicad/api/proto/common/commands/editor_commands.proto:373) (`ClearSelection`)
  - [`kicad/api/proto/common/commands/editor_commands.proto:424`](/Users/milindsharma/Developer/kicad-oss/kicad-ipc-rs/kicad/api/proto/common/commands/editor_commands.proto:424) (`SavedSelectionResponse`)
- Client flow:
  - [`src/client.rs:1067`](/Users/milindsharma/Developer/kicad-oss/kicad-ipc-rs/src/client.rs:1067) (`get_selection_raw`)
  - [`src/client.rs:1083`](/Users/milindsharma/Developer/kicad-oss/kicad-ipc-rs/src/client.rs:1083) (`get_selection_details`)
  - [`src/client.rs:1089`](/Users/milindsharma/Developer/kicad-oss/kicad-ipc-rs/src/client.rs:1089) (`get_selection`)
  - [`src/client.rs:1046`](/Users/milindsharma/Developer/kicad-oss/kicad-ipc-rs/src/client.rs:1046) (`get_selection_summary`)
  - [`src/client.rs:1123`](/Users/milindsharma/Developer/kicad-oss/kicad-ipc-rs/src/client.rs:1123) (`add_to_selection`)
  - [`src/client.rs:1153`](/Users/milindsharma/Developer/kicad-oss/kicad-ipc-rs/src/client.rs:1153) (`clear_selection`)
  - [`src/client.rs:1187`](/Users/milindsharma/Developer/kicad-oss/kicad-ipc-rs/src/client.rs:1187) (`remove_from_selection`)
  - [`src/client.rs:1849`](/Users/milindsharma/Developer/kicad-oss/kicad-ipc-rs/src/client.rs:1849) (`get_selection_as_string`)
  - [`src/client.rs:2384`](/Users/milindsharma/Developer/kicad-oss/kicad-ipc-rs/src/client.rs:2384) (`summarize_selection`)
  - [`src/client.rs:2401`](/Users/milindsharma/Developer/kicad-oss/kicad-ipc-rs/src/client.rs:2401) (`summarize_item_details`)
  - [`src/client.rs:3108`](/Users/milindsharma/Developer/kicad-oss/kicad-ipc-rs/src/client.rs:3108) (`decode_pcb_item`)
- Public model bottleneck:
  - [`src/model/board.rs:389`](/Users/milindsharma/Developer/kicad-oss/kicad-ipc-rs/src/model/board.rs:389) onward (`Pcb*` structs + `PcbItem`)
  - [`src/model/common.rs:194`](/Users/milindsharma/Developer/kicad-oss/kicad-ipc-rs/src/model/common.rs:194) (`SelectionSummary`)
  - [`src/model/common.rs:203`](/Users/milindsharma/Developer/kicad-oss/kicad-ipc-rs/src/model/common.rs:203) (`SelectionItemDetail`)
- Relevant proto item schemas:
  - [`src/proto/generated/kiapi.board.types.rs:19`](/Users/milindsharma/Developer/kicad-oss/kicad-ipc-rs/src/proto/generated/kiapi.board.types.rs:19) (`Track`)
  - [`src/proto/generated/kiapi.board.types.rs:39`](/Users/milindsharma/Developer/kicad-oss/kicad-ipc-rs/src/proto/generated/kiapi.board.types.rs:39) (`Arc`)
  - [`src/proto/generated/kiapi.board.types.rs:227`](/Users/milindsharma/Developer/kicad-oss/kicad-ipc-rs/src/proto/generated/kiapi.board.types.rs:227) (`Via`)
  - [`src/proto/generated/kiapi.board.types.rs:305`](/Users/milindsharma/Developer/kicad-oss/kicad-ipc-rs/src/proto/generated/kiapi.board.types.rs:305) (`Pad`)
  - [`src/proto/generated/kiapi.board.types.rs:420`](/Users/milindsharma/Developer/kicad-oss/kicad-ipc-rs/src/proto/generated/kiapi.board.types.rs:420) (`Zone`)
  - [`src/proto/generated/kiapi.board.types.rs:520`](/Users/milindsharma/Developer/kicad-oss/kicad-ipc-rs/src/proto/generated/kiapi.board.types.rs:520) (`Dimension`)
  - [`src/proto/generated/kiapi.board.types.rs:580`](/Users/milindsharma/Developer/kicad-oss/kicad-ipc-rs/src/proto/generated/kiapi.board.types.rs:580) (`Group`)
  - [`src/proto/generated/kiapi.board.types.rs:705`](/Users/milindsharma/Developer/kicad-oss/kicad-ipc-rs/src/proto/generated/kiapi.board.types.rs:705) (`FootprintInstance`)
  - [`src/proto/generated/kiapi.common.types.rs:541`](/Users/milindsharma/Developer/kicad-oss/kicad-ipc-rs/src/proto/generated/kiapi.common.types.rs:541) (`Text`)
  - [`src/proto/generated/kiapi.common.types.rs:554`](/Users/milindsharma/Developer/kicad-oss/kicad-ipc-rs/src/proto/generated/kiapi.common.types.rs:554) (`TextBox`)
  - [`src/proto/generated/kiapi.common.types.rs:634`](/Users/milindsharma/Developer/kicad-oss/kicad-ipc-rs/src/proto/generated/kiapi.common.types.rs:634) (`GraphicShape`)

## Current State: What Is Lossy vs Not

### Not lossy

- `get_selection_raw` returns `SelectionResponse.items` directly (`Vec<Any>`). No internal field drop.
- `*_selection_raw` variants for add/remove/clear preserve raw payload when server returns `SelectionResponse`.

### Lossy API layers

- `get_selection_summary`: compresses all item payloads into counts by `type_url`.
- `get_selection_details`: flattens into human/debug string + byte length; no structured fields.
- `get_selection`: decodes into reduced `PcbItem` models with many fields omitted.
- `add_to_selection` / `remove_from_selection` / `clear_selection`: typed wrappers return summary only.
- `get_selection_as_string`: drops `SavedSelectionResponse.ids`; returns `contents` only.
- `GetSelection.types` filter exists in proto, but no public method exposes it (always empty in current code).

## Loss Inventory by Item Type (proto -> public typed)

- `Track`: drops `locked`.
- `Arc`: drops `locked`.
- `Via`: drops `locked`; keeps only shallow `pad_stack` info (layer span + drill start/end), drops drill geometry and advanced padstack settings.
- `Pad`: drops `locked`, full `pad_stack`, clearance override, die length/delay, symbol pin metadata.
- `FootprintInstance`: keeps id/ref/pos/orientation/layer/pad_count; drops definition internals, fields (`value`, `datasheet`, `description`), attributes/overrides, symbol linkage metadata.
- `BoardGraphicShape`: keeps geometry kind as string only; drops structured geometry + graphic attributes.
- `BoardText` / `BoardTextBox`: keep body text only; drop position/box, style attributes, hyperlink, lock/knockout.
- `Zone`: keeps coarse stats (type/counts/filled); drops outline, settings, border, layer properties, priority.
- `Dimension`: keeps text/layer/style string only; drops detailed unit/precision/style geometry and overrides.
- `Group`: keeps `item_count`; drops actual item id list.

## Extra Coverage Gaps

- `decode_pcb_item` supports 12 board item payload types only. Other PCB object types can appear as `Unknown` in typed API.
- `proto` module is crate-private. Consumers get `Any` bytes, not generated proto structs from this crate.

## Implementation Plan (follow in order)

### Phase 1: additive APIs, zero breakage

1. Add richer selection-return models in `src/model/common.rs`:
   - `SelectionStringDump { ids: Vec<String>, contents: String }`
   - `SelectionMutationResult { items: Vec<Any>, summary: SelectionSummary }` or equivalent typed struct without reducing to summary-only.
2. Add new `KiCadClient` methods in `src/client.rs`:
   - `get_selection_with_types(type_codes: Vec<i32>) -> Vec<PcbItem>` and raw/details variants.
   - `get_selection_string_dump() -> SelectionStringDump` (keep existing `get_selection_as_string` as convenience).
   - Rich mutation variants for add/remove/clear that expose returned items, not summary only.
3. Export new models via `src/lib.rs`.
4. Add blocking mirror methods in `src/blocking.rs`.

### Phase 2: reduce typed-model loss

1. Expand `Pcb*` structs in `src/model/board.rs` with additive optional fields (no removals).
2. Update `decode_pcb_item` mapping in `src/client.rs` to fill new fields.
3. Prefer structured enums over stringified debug fields where possible:
   - graphic geometry
   - dimension style
4. Preserve backward compatibility:
   - existing fields remain
   - new fields optional/defaultable

### Phase 3: unhandled item kinds

1. Add typed support for additional PCB object payload types if proto types exist in generated files.
2. If unavailable in proto snapshot, keep `Unknown` fallback; include `type_url` + `raw_len`.

### Phase 4: docs/tests/regression

1. Unit tests in `src/client.rs`:
   - new selection filter path
   - new response models keep previously dropped fields
   - backward compatibility on old methods
2. Update docs:
   - `README.md` API table
   - `docs/PCB_SELECTION_DEEP_DUMP.md` sequence updates
3. Validation commands:
   - `cargo fmt --all`
   - `cargo test`
   - `cargo test --features blocking`

## Decision Log Needed Before Coding

- Whether to expose proto-level structs publicly (`pub mod proto`) vs keep custom models only.
- Whether `get_selection` should stay “compact model” and new methods be “full model” (recommended).
- Naming:
  - keep existing methods untouched
  - add explicit `*_full`/`*_rich` APIs for clarity.

## Acceptance Criteria

- No breaking changes in existing method signatures.
- New selection APIs expose:
  - selection type filtering
  - `SavedSelectionResponse.ids`
  - non-summary mutation payload access
  - materially more per-item structured data than current `PcbItem`.
- Existing examples still compile; add one new example showcasing rich selection extraction.
