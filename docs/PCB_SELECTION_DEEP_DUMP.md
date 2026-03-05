# PCB Selection Deep Dump

How to extract maximum data for current PCB selection using `kicad-ipc-rs` bindings.

## Script

Use:

- `examples/selection_deep_dump.rs`

Run:

```bash
RUSTFLAGS='-Awarnings' cargo run -q --features blocking --example selection_deep_dump
```

## What It Collects

- Selection summary:
  - total selected item count
  - per-type counts (`type_url`)
- Decoded selected items (`get_selection(Vec::new())`)
- Raw selected payload metadata (`get_selection_raw(Vec::new())`)
- Human-readable selection rows (`get_selection_details(Vec::new())`)
- Item IDs for all selected items
- Item bounding boxes (`get_item_bounding_boxes`)
- Footprint-level fields:
  - reference
  - UUID
  - position (nm)
  - orientation (deg)
  - layer
  - pad count
- Designator/value pairs from selection text (`get_selection_as_string().contents`)
- Selection text dump item IDs (`get_selection_as_string().ids`)
- Pad-level net rows for selected footprints (`get_pad_netlist`)
- Net graph among selected references (`net -> refs`)
- Net name to net code mapping (`get_nets`)

## API Sequence (Bindings)

- `KiCadClientBlocking::connect`
- `ping`
- `get_version`
- `get_selection_summary(Vec::new())`
- `get_selection(Vec::new())`
- `get_selection_raw(Vec::new())`
- `get_selection_details(Vec::new())`
- `get_item_bounding_boxes`
- `get_selection_as_string`
- `get_pad_netlist`
- `get_nets`
- `get_items_by_net` (best path; may be unavailable)
- fallback: `get_items_by_type_codes` per type code + local net-name filter

## Route/Trace Discovery Logic

Primary:

- Query `get_items_by_net` with types:
  - `KOT_PCB_TRACE`
  - `KOT_PCB_VIA`
  - `KOT_PCB_ARC`
  - `KOT_PCB_ZONE`
  - `KOT_PCB_PAD`
  - `KOT_PCB_SHAPE`

Fallback when `GetItemsByNet` is unsupported:

- Query each type separately with `get_items_by_type_codes(vec![type_code])`
- Filter returned items locally by `item.net.name in selected_net_names`

## Known KiCad 10.0.0-rc1 Behavior Seen

- `kiapi.board.commands.GetItemsByNet` can return `AS_UNHANDLED`.
- Script handles this and continues with fallback scan.
- On current session, fallback returned pad-connected items; no extra tracks/vias/arcs were returned for selected net names.

## Notes

- Script is read-only.
- Retries are built in for transient API unhandled/timeouts.
- Output is verbose by design, intended for debugging and data mining.
