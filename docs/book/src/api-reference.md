# API Reference

Primary API docs live on docs.rs:

- [kicad-ipc-rs API Reference](https://docs.rs/kicad-ipc-rs)

Key items:

- `KiCadClient` (async)
- `KiCadClientBlocking` (`blocking` feature)
- `KiCadError`
- Typed models under `model::*`

Selection API notes:

- `get_selection_*` methods now take `type_codes: Vec<i32>` (`Vec::new()` means no filter).
- `add_to_selection`, `remove_from_selection`, `clear_selection` return `SelectionMutationResult` (decoded items + summary).
- `get_selection_as_string` returns `SelectionStringDump` (`ids` + `contents`).
