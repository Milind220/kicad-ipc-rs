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

Editable item helpers:

- `get_editable_items_by_id(...)`
- `get_editable_items_by_type_codes(...)`
- `create_editable_items(...)`
- `update_editable_items(...)`

Use `EditablePcbItem` when you need to fetch existing board items, mutate fields like layer or position, and write them back through KiCad IPC without hand-building protobuf `Any` payloads. The editable wrappers expose `proto()`, `proto_mut()`, and `into_proto()` as advanced escape hatches when typed helpers are not enough.

Selection API notes:

- `get_selection_*` methods now take `type_codes: Vec<i32>` (`Vec::new()` means no filter).
- `add_to_selection`, `remove_from_selection`, `clear_selection` return `SelectionMutationResult` (decoded items + summary).
- `get_selection_as_string` returns `SelectionStringDump` (`ids` + `contents`).
