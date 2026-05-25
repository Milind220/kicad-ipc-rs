# API Reference

Primary API docs live on docs.rs:

- [kicad-ipc-rs API Reference](https://docs.rs/kicad-ipc-rs)

Key items:

- `KiCadClient` (async)
- `KiCadClientBlocking` (`blocking` feature)
- `KiCadError`
- Typed models under `model::*`

PCB item API layers:

- Raw IPC: `*_raw` methods return `prost_types::Any` payloads for direct protobuf interop.
- Read model: `PcbItem` and related `Pcb*` structs are lightweight decoded models for inspection.
- Editable model: `EditablePcbItem` and typed wrappers preserve the full protobuf payload for mutate/update workflows.

Raw command escape hatch:

- `KiCadClient::send_raw_command(...)`
- `KiCadClientBlocking::send_raw_command(...)` (`blocking` feature)

Use this when KiCad exposes a protobuf command that does not yet have a typed helper.

Editable item helpers:

- `get_editable_items_by_id(...)`
- `get_editable_items_by_type_codes(...)`
- `create_editable_items(...)`
- `update_editable_items(...)`

Use `EditablePcbItem` when you need to fetch existing board items, mutate fields like layer or position, and write them back through KiCad IPC without hand-building protobuf `Any` payloads. The editable wrappers expose `proto()`, `proto_mut()`, and `into_proto()` as advanced escape hatches when typed helpers are not enough.

Typed board text helpers:

- `create_board_text(...)`
- `create_board_texts(...)`
- `create_board_text_in_container(...)`
- `create_board_texts_in_container(...)`

These helpers build `kiapi.board.types.BoardText` payloads and send typed `CreateItems`, matching kicad-python's direct `BoardText` creation path. Prefer them for silkscreen and board text creation instead of `parse_and_create_items_from_string(...)`.

All-PCB item reads:

- `get_all_pcb_items_raw(...)`
- `get_all_pcb_items_details(...)`
- `get_all_pcb_items(...)`

These methods use one combined `GetItems` request for the known PCB object classes, then bucket returned payloads by KiCad item type. If KiCad returns a payload type the crate cannot map, the call returns `KiCadError::InvalidResponse` instead of silently dropping it.

Deletion notes:

- `delete_items(...)` returns ids reported by KiCad as deleted.
- KiCad 10.0.x may acknowledge `DeleteItems` with no per-item rows. In that case `delete_items(...)` returns the requested ids after request success; treat them as accepted by KiCad, not independently verified deleted.

Object and layer lookup helpers:

- `PcbObjectTypeCode::from_name(...)` accepts proto names and friendly names like `trace`, `track`, `footprint`, `pad`, `text`, and `silkscreen-text`.
- `PcbObjectTypeCode::new_trace()`, `new_pad()`, `new_text()`, and related constructors avoid hardcoded object type numbers.
- `BoardLayerInfo::id_from_name(...)` and `canonical_name_for_id(...)` resolve KiCad board layer names such as `F.SilkS` and protobuf names such as `BL_F_SilkS`.

Selection API notes:

- `get_selection_*` methods now take `type_codes: Vec<i32>` (`Vec::new()` means no filter).
- `add_to_selection`, `remove_from_selection`, `clear_selection` return `SelectionMutationResult` (decoded items + summary).
- `get_selection_as_string` returns `SelectionStringDump` (`ids` + `contents`).
