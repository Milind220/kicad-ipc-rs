# kicad-ipc-rs

[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/Milind220/kicad-ipc-rust)

Control KiCad programmatically from Rust with an async-first API and optional blocking wrappers.
- **KiCad 11 groundwork branch** generated from KiCad `10.99.0-991-gd3bb3db575`
- **Raw KiCad 11 command helpers** for new project, editor, board, schematic, and job command families
- **KiCad 10 wrapper coverage preserved** while KiCad 11 typed wrappers are tracked separately
- **Type-safe PCB item manipulation** with ergonomic Rust models
- **Both async and blocking APIs** for any application architecture
- **Checked-in protobuf output** so consumers do not need a KiCad source checkout

## Status

Unstable KiCad 11 / 10.99 support branch. The checked-in protos are generated from KiCad `origin/master` at `d3bb3db575303c86edd68d526258d6e5b9294bc9`, described as `10.99.0-991-gd3bb3db575`.

Existing KiCad 10 typed wrappers remain in place, but new KiCad 11 APIs are exposed first through generated protobuf types plus raw command helpers. Do not treat this branch as stable KiCad 11 API support yet.

- Async API (default): primary supported surface
- Sync/blocking wrapper API (`feature = "blocking"`): wraps async calls on a dedicated Tokio runtime thread
- Low-level proto/raw API: `kicad_ipc_rs::proto`, `kicad_ipc_rs::commands::*`, and `KiCadClient::send_raw_command`

## Breaking Changes (Unreleased)

The current unreleased branch includes API behavior changes that are **breaking for 0.4.x users** (pre-1.0 semver: breaking changes are released in a new **minor** version).

Migration notes:

- `TitleBlockInfo.comments` now preserves fixed `comment1..comment9` slot ordering and internal empty gaps when round-tripping through `set_title_block_info` / `get_title_block_info`.
- `get_items_by_net` now documents KiCad 10.0.1 behavior explicitly: net **names** are authoritative; numeric net codes are legacy compatibility fields.

## Prerequisites
- **Rust 1.70+** (edition 2021)
- **KiCad 11 development / 10.99** running with the IPC API enabled, or headless `kicad-cli api-server`
- The `nng` transport library is bundled automatically via [nng-rs](https://crates.io/crates/nng)

### Enabling the KiCad IPC API
1. Open KiCad → **Preferences** → **Plugins**
2. Check **Enable IPC API**  
3. Restart KiCad

The API socket path is auto-detected. Override with `KICAD_API_SOCKET` if needed.

## Usage

### Async API (Default)

Add to `Cargo.toml`:

```toml
[dependencies]
kicad-ipc-rs = "0.4.1"
tokio = { version = "1", features = ["macros", "rt"] }
```

Connect and query KiCad:

```rust
use kicad_ipc_rs::KiCadClient;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), kicad_ipc_rs::KiCadError> {
    let client = KiCadClient::connect().await?;
    
    // Get KiCad version info
    let version = client.get_version().await?;
    println!("Connected to KiCad {}", version.full_version);
    
    // Check if a board is open
    if client.has_open_board().await? {
        // Get all nets in the current board
        let nets = client.get_nets().await?;
        println!("Found {} nets", nets.len());
        
        // Get all tracks on the board
        let tracks = client.get_items_by_type_codes(vec![
            kicad_ipc_rs::PcbObjectTypeCode::new_trace()
        ]).await?;
        println!("Found {} tracks", tracks.len());
    }
    
    Ok(())
}
```

### Sync API (Blocking)

Enable the `blocking` feature for synchronous applications:

```toml
[dependencies]
kicad-ipc-rs = { version = "0.4.1", features = ["blocking"] }
```

```rust
use kicad_ipc_rs::KiCadClientBlocking;

fn main() -> Result<(), kicad_ipc_rs::KiCadError> {
    let client = KiCadClientBlocking::connect()?;
    
    // Get all nets and find unconnected ones
    let nets = client.get_nets()?;
    let unconnected: Vec<_> = nets
        .iter()
        .filter(|n| n.name == "unconnected")
        .collect();
    
    println!("Found {} unconnected nets", unconnected.len());
    Ok(())
}
```

### Making Changes to PCBs

All board modifications use commit sessions for safety:

```rust
use kicad_ipc_rs::{KiCadClient, CommitAction};

async fn add_track(client: &KiCadClient) -> Result<(), kicad_ipc_rs::KiCadError> {
    // Start a commit session
    let commit = client.begin_commit().await?;
    
    // Create items (tracks, vias, footprints, etc.)
    let items = vec![/* your PcbItem instances */];
    let created_ids = client.create_items(items).await?;
    
    // Commit the changes
    client.end_commit(
        commit.id,
        CommitAction::Commit,
        "Added new track"
    ).await?;
    
    Ok(())
}
```

For in-place editing flows, fetch editable items, mutate them, then write them back:

```rust
use kicad_ipc_rs::EditablePcbItem;

let mut items = client.get_editable_items_by_id(ids).await?;
let back_cu = client.get_active_layer().await?.id;

for item in &mut items {
    match item {
        EditablePcbItem::Track(track) => track.set_layer_id(back_cu),
        EditablePcbItem::BoardText(text) => text.set_layer_id(back_cu),
        EditablePcbItem::Zone(zone) => zone.set_layer_ids(vec![back_cu]),
        _ => {}
    }
}

client.update_editable_items(items).await?;
```

### PCB Item Model Layers

- **Raw IPC layer**: `prost_types::Any` payloads from KiCad commands (`*_raw` APIs).
- **Read model layer**: `PcbItem` for inspection/analysis when you do not need mutation.
- **Editable model layer**: `EditablePcbItem` for ergonomic mutate/update workflows.

`EditablePcbItem` wrappers also expose `proto()` / `proto_mut()` / `into_proto()` as advanced
escape hatches when you need direct protobuf access.

## Examples
Run the included examples against a running KiCad instance:

```bash
# Minimal connection + version check
cargo run --example hello_kicad --features blocking

# Inspect board nets, layers, and origin
cargo run --example board_inspector --features blocking

# Deep-dive into current PCB selection
cargo run --example selection_deep_dump --features blocking
```

See the [examples/](examples/) directory for full source.

## KiCad Version Compatibility

This branch targets unstable KiCad 11 / 10.99 IPC bindings. The proto snapshot is `10.99.0-991-gd3bb3db575`; see `KICAD_API_VERSION`.

KiCad 10 wrapper behavior is preserved where the KiCad 11 schema still supports it. New KiCad 11 command families are currently raw-first; typed wrappers are tracked in [docs/KICAD_11_IPC_PLAN.md](docs/KICAD_11_IPC_PLAN.md).

## KiCad 11 Raw IPC Surface

The unstable branch exposes generated protobuf modules and command packers:

- `kicad_ipc_rs::proto::kiapi::*`: checked-in prost output for the KiCad snapshot.
- `kicad_ipc_rs::commands::project`: `OpenDocument`, `CloseDocument`, `SaveDocument`.
- `kicad_ipc_rs::commands::editor`: page settings get/set.
- `kicad_ipc_rs::commands::board`: board design rules and board export jobs.
- `kicad_ipc_rs::commands::schematic`: hierarchy/netlist and schematic export jobs.
- `KiCadClient::send_raw_command`: sends a pre-packed `prost_types::Any` command and returns the raw response payload.

## KiCad v10.0.1 Wrapped API Reference

The previously wrapped KiCad v10.0.1 command set remains available on this branch:

### Section Coverage
| Section | Commands | Coverage |
| --- | ---: | ---: |
| Common (base) | 6 | 100% |
| Common editor/document | 24 | 100% |
| Project manager | 5 | 100% |
| Board editor (PCB) | 24 | 100% |
| **Total** | **59** | **100%** |

### Command Reference
**Common (base)**

| KiCad Command | Rust API |
| --- | --- |
| `Ping` | `KiCadClient::ping` |
| `GetVersion` | `KiCadClient::get_version` |
| `GetKiCadBinaryPath` | `KiCadClient::get_kicad_binary_path` |
| `GetTextExtents` | `KiCadClient::get_text_extents` |
| `GetTextAsShapes` | `KiCadClient::get_text_as_shapes` |
| `GetPluginSettingsPath` | `KiCadClient::get_plugin_settings_path` |

**Common editor/document**

| KiCad Command | Rust API |
| --- | --- |
| `RefreshEditor` | `KiCadClient::refresh_editor` |
| `GetOpenDocuments` | `KiCadClient::get_open_documents`, `get_current_project_path`, `has_open_board` |
| `SaveDocument` | `KiCadClient::save_document` |
| `SaveCopyOfDocument` | `KiCadClient::save_copy_of_document` |
| `RevertDocument` | `KiCadClient::revert_document` |
| `RunAction` | `KiCadClient::run_action` |
| `BeginCommit` / `EndCommit` | `KiCadClient::begin_commit`, `end_commit` |
| `CreateItems` | `KiCadClient::create_items` |
| `GetItems` | `KiCadClient::get_items_by_type_codes`, `get_all_pcb_items`, `get_pad_netlist` |
| `GetItemsById` | `KiCadClient::get_items_by_id` |
| `UpdateItems` | `KiCadClient::update_items` |
| `DeleteItems` | `KiCadClient::delete_items` |
| `GetBoundingBox` | `KiCadClient::get_item_bounding_boxes` |
| `GetSelection` | `KiCadClient::get_selection`, `get_selection_summary`, `get_selection_details` |
| `AddToSelection` / `RemoveFromSelection` / `ClearSelection` | `KiCadClient::add_to_selection`, `remove_from_selection`, `clear_selection` |
| `HitTest` | `KiCadClient::hit_test_item` |
| `GetTitleBlockInfo` / `SetTitleBlockInfo` | `KiCadClient::get_title_block_info`, `set_title_block_info` |
| `SaveDocumentToString` | `KiCadClient::get_board_as_string` |
| `SaveSelectionToString` | `KiCadClient::get_selection_as_string` |
| `ParseAndCreateItemsFromString` | `KiCadClient::parse_and_create_items_from_string` |

**Project manager**
| KiCad Command | Rust API |
| --- | --- |
| `GetNetClasses` / `SetNetClasses` | `KiCadClient::get_net_classes`, `set_net_classes` |
| `ExpandTextVariables` | `KiCadClient::expand_text_variables` |
| `GetTextVariables` / `SetTextVariables` | `KiCadClient::get_text_variables`, `set_text_variables` |

**Board editor (PCB)**

| KiCad Command | Rust API |
| --- | --- |
| `GetBoardStackup` / `UpdateBoardStackup` | `KiCadClient::get_board_stackup`, `update_board_stackup` |
| `GetBoardEnabledLayers` / `SetBoardEnabledLayers` | `KiCadClient::get_board_enabled_layers`, `set_board_enabled_layers` |
| `GetGraphicsDefaults` | `KiCadClient::get_graphics_defaults` |
| `GetBoardOrigin` / `SetBoardOrigin` | `KiCadClient::get_board_origin`, `set_board_origin` |
| `GetNets` | `KiCadClient::get_nets` |
| `GetItemsByNet` / `GetItemsByNetClass` | `KiCadClient::get_items_by_net`, `get_items_by_net_class` |
| `GetConnectedItems` | `KiCadClient::get_connected_items` |
| `GetNetClassForNets` | `KiCadClient::get_netclass_for_nets` |
| `RefillZones` | `KiCadClient::refill_zones` |
| `GetPadShapeAsPolygon` | `KiCadClient::get_pad_shape_as_polygon` |
| `CheckPadstackPresenceOnLayers` | `KiCadClient::check_padstack_presence_on_layers` |
| `InjectDrcError` | `KiCadClient::inject_drc_error` |
| `GetVisibleLayers` / `SetVisibleLayers` | `KiCadClient::get_visible_layers`, `set_visible_layers` |
| `GetActiveLayer` / `SetActiveLayer` | `KiCadClient::get_active_layer`, `set_active_layer` |
| `GetBoardLayerName` | `KiCadClient::get_board_layer_name` |
| `GetBoardEditorAppearanceSettings` / `SetBoardEditorAppearanceSettings` | `KiCadClient::get_board_editor_appearance_settings`, `set_board_editor_appearance_settings` |
| `InteractiveMoveItems` | `KiCadClient::interactive_move_items` |

> `GetItemsByNet` guidance (KiCad 10.0.1): net names are authoritative; net codes are legacy compatibility fields.

## Documentation
- **Guide**: [https://milind220.github.io/kicad-ipc-rs/](https://milind220.github.io/kicad-ipc-rs/)
- **API Reference**: [docs.rs/kicad-ipc-rs](https://docs.rs/kicad-ipc-rs)

## Protobuf Source

This crate ships checked-in Rust protobuf output under `src/proto/generated/`.

- Consumers do **not** need KiCad source checkout or git submodules
- Maintainers regenerate bindings from KiCad upstream via the `kicad` git submodule
- Current proto pin: KiCad `10.99.0-991-gd3bb3db575` (`KICAD_API_VERSION = 10.99.0-991-gd3bb3db575`)
Maintainer refresh flow:

```bash
git submodule update --init --recursive
./scripts/regenerate-protos.sh
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development workflow and commit conventions.

Issues and PRs welcome!

## License

MIT
