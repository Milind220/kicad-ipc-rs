# Usage Patterns

This chapter targets repeatable integration patterns for tool builders and code generators.

## Pattern: Cheap Health Check

Use at process startup to validate socket + auth + server liveness.

```rust,no_run
use kicad_ipc_rs::KiCadClient;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), kicad_ipc_rs::KiCadError> {
    let client = KiCadClient::connect().await?;
    client.ping().await?;
    Ok(())
}
```

## Pattern: Read-only Query Pipeline

Recommended order for board-aware reads:

1. `get_open_documents()`
2. `get_nets()`
3. `get_items_by_net(...)` or `get_items_by_type_codes(...)`

Reason: fail fast on document state before expensive item traversal.

## Pattern: Safe Write Session

Use begin/end commit around mutating commands.

1. `begin_commit(...)`
2. `create_items(...)` / `update_items(...)` / `update_editable_items(...)` / `delete_items(...)`
3. `end_commit(..., CommitAction::Commit, ...)`

If errors mid-flight: close with `CommitAction::Abort`/`Drop` per flow.

## Pattern: Editable Item Mutation

Use `EditablePcbItem` when you want to round-trip existing board items without manually decoding and packing protobuf `Any` payloads.

```rust,no_run
use kicad_ipc_rs::{CommitAction, EditablePcbItem, KiCadClient};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), kicad_ipc_rs::KiCadError> {
    let client = KiCadClient::connect().await?;
    let commit = client.begin_commit().await?;

    let mut items = client.get_editable_items_by_type_codes(vec![
        KiCadClient::pcb_object_type_codes()
            .iter()
            .find(|entry| entry.name == "KOT_PCB_TRACE")
            .expect("trace object type should exist")
            .code,
    ]).await?;

    for item in &mut items {
        if let EditablePcbItem::Track(track) = item {
            track.set_layer_id(0);
        }
    }

    client.update_editable_items(items).await?;
    client
        .end_commit(commit, CommitAction::Commit, "move tracks to layer")
        .await?;

    Ok(())
}
```

Prefer typed wrapper methods like `set_layer_id`, `set_layer_ids`, and position setters. Use `proto_mut()` only for advanced cases where the typed editable API does not yet expose the field you need.

## Common Pitfalls

| Pitfall | Symptom | Avoidance |
| --- | --- | --- |
| Assume KiCad always running | connect errors at startup | explicit prereq check + `ping()` |
| Skip open-document check | downstream command failures | call `get_open_documents()` first |
| Mix sync + async API unintentionally | duplicate runtime ownership | pick one surface per process |
| Fire write commands without commit session | partial or rejected mutations | always bracket writes with commit APIs |
| Hardcode unsupported commands | `AS_UNHANDLED` at runtime | map/handle `RunActionStatus` and runtime flags |
| Use read models for mutation | no way to write the item back losslessly | fetch `EditablePcbItem` instead of `PcbItem` |

## Async vs Blocking Selection

| Requirement | Preferred API |
| --- | --- |
| Tokio app / async daemon | `KiCadClient` |
| Existing sync binary | `KiCadClientBlocking` |
| Lowest integration friction for scripts | `KiCadClientBlocking` + CLI |

## Reliability Checklist

- Set explicit `client_name` for traceability.
- Keep request timeout defaults unless measured need.
- Handle transport + protocol errors as recoverable boundary.
- Use typed wrappers when available; drop to raw only when needed.
