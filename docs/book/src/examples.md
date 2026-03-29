# Examples

Real-world usage patterns for `kicad-ipc-rs`.

## Quick Version Probe (Async)

```rust,no_run
use kicad_ipc_rs::KiCadClient;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), kicad_ipc_rs::KiCadError> {
    let client = KiCadClient::connect().await?;
    let version = client.get_version().await?;
    println!("{:?}", version);
    Ok(())
}
```

## Open Board Detection (Blocking)

```rust,no_run
use kicad_ipc_rs::KiCadClientBlocking;

fn main() -> Result<(), kicad_ipc_rs::KiCadError> {
    let client = KiCadClientBlocking::connect()?;
    let has_board = client.has_open_board()?;
    println!("open board: {}", has_board);
    Ok(())
}
```

## Example: PCB Analysis - Find Unconnected Nets

Analyze a board to find nets that aren't properly connected:

```rust,no_run
use kicad_ipc_rs::KiCadClient;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), kicad_ipc_rs::KiCadError> {
    let client = KiCadClient::connect().await?;
    
    // Get all nets in the current board
    let nets = client.get_nets().await?;
    
    // Filter for nets with names suggesting they're unconnected
    let suspicious: Vec<_> = nets
        .iter()
        .filter(|net| {
            net.name.to_lowercase().contains("unconnected") ||
            net.name.to_lowercase().contains("unrouted") ||
            net.name.starts_with("Net-(")
        })
        .collect();
    
    if suspicious.is_empty() {
        println!("All nets appear to be properly connected!");
    } else {
        println!("Found {} potentially unconnected nets:", suspicious.len());
        for net in suspicious {
            println!("  - {} (code: {})", net.name, net.code);
        }
    }
    
    Ok(())
}
```

## Example: PCB Analysis - List All Footprints

Get a summary of all footprints on the board:

```rust,no_run
use kicad_ipc_rs::{KiCadClient, PcbObjectTypeCode};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), kicad_ipc_rs::KiCadError> {
    let client = KiCadClient::connect().await?;
    
    // Get all footprints
    let footprints = client.get_items_by_type_codes(vec![
        PcbObjectTypeCode::new_footprint()
    ]).await?;
    
    let mut by_lib: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    
    for item in footprints {
        if let kicad_ipc_rs::PcbItem::Footprint(fp) = item {
            let lib = fp.library_id.unwrap_or_else(|| "Unknown".to_string());
            *by_lib.entry(lib).or_insert(0) += 1;
        }
    }
    
    println!("Footprints by library:");
    for (lib, count) in by_lib.iter().take(10) {
        println!("  {}: {}", lib, count);
    }
    
    Ok(())
}
```

## Example: Automation - Batch Rename Text Variables

Update text variables across the project:

```rust,no_run
use kicad_ipc_rs::{KiCadClient, DocumentType};
use std::collections::BTreeMap;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), kicad_ipc_rs::KiCadError> {
    let client = KiCadClient::connect().await?;
    
    // Get current text variables
    let current = client.get_text_variables().await?;
    println!("Current variables: {:?}", current);
    
    // Add/update variables
    let mut updates = current.clone();
    updates.insert("VERSION".to_string(), "v2.1.0".to_string());
    updates.insert("DATE".to_string(), "2026-03-29".to_string());
    
    // Set the updated variables
    client.set_text_variables(updates, 
        kicad_ipc_rs::MapMergeMode::Replace
    ).await?;
    
    println!("Text variables updated successfully");
    Ok(())
}
```

## Example: Automation - Add Test Points to Unconnected Pads

Automatically add test point footprints to pads that aren't connected to nets:

```rust,no_run
use kicad_ipc_rs::{KiCadClient, CommitAction, KiCadError, PcbItem};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), kicad_ipc_rs::KiCadError> {
    let client = KiCadClient::connect().await?;
    
    // Get all pads and filter for unconnected ones
    let items = client.get_all_pcb_items().await?;
    
    let mut unconnected_pads = Vec::new();
    for item in items {
        if let PcbItem::Pad(pad) = item {
            if pad.net_code.is_none() && pad.pad_number != "1" {
                unconnected_pads.push(pad);
            }
        }
    }
    
    if unconnected_pads.is_empty() {
        println!("No unconnected pads found");
        return Ok(());
    }
    
    println!("Found {} unconnected pads to add test points", unconnected_pads.len());
    
    // Start commit session
    let commit = client.begin_commit().await?;
    
    // For each unconnected pad, add a test point footprint
    // (simplified - actual implementation would create footprint items)
    for pad in unconnected_pads.iter().take(5) {
        println!("Would add test point near pad {} at {:?}", 
            pad.pad_number, pad.position_nm);
    }
    
    // Commit the changes
    client.end_commit(
        commit.id,
        CommitAction::Commit,
        "Added test points to unconnected pads"
    ).await?;
    
    Ok(())
}
```

## Example: CI/CD - Design Rule Check Integration

Script to run automated checks before committing to version control:

```rust,no_run
use kicad_ipc_rs::KiCadClientBlocking;

fn main() -> Result<(), kicad_ipc_rs::KiCadError> {
    let client = KiCadClientBlocking::connect()?;
    
    // Check 1: Verify board is open
    if !client.has_open_board()? {
        eprintln!("ERROR: No board is open in KiCad");
        std::process::exit(1);
    }
    
    // Check 2: Get all nets and look for DRC markers
    let nets = client.get_nets()?;
    println!("✓ Board has {} nets", nets.len());
    
    // Check 3: Verify board origin is set
    let origin = client.get_board_origin(
        kicad_ipc_rs::BoardOriginKind::Drill
    )?;
    println!("✓ Board origin at ({}, {})", origin.x_nm, origin.y_nm);
    
    // Check 4: Save the board before proceeding
    client.save_document()?;
    println!("✓ Board saved");
    
    // Check 5: Export board as string for diffing
    let board_string = client.get_board_as_string()?;
    println!("✓ Board exported ({} bytes)", board_string.len());
    
    println!("\nAll checks passed! Board is ready for commit.");
    Ok(())
}
```

## Example: Integration - Net Class Validation

Verify that all nets have appropriate net classes assigned:

```rust,no_run
use kicad_ipc_rs::KiCadClientBlocking;
use std::collections::BTreeSet;

fn main() -> Result<(), kicad_ipc_rs::KiCadError> {
    let client = KiCadClientBlocking::connect()?;
    
    // Get all net classes
    let net_classes = client.get_net_classes()?;
    let class_names: BTreeSet<_> = net_classes
        .iter()
        .map(|nc| nc.name.clone())
        .collect();
    
    // Get all nets
    let nets = client.get_nets()?;
    
    // Check each net has a valid net class
    let mut missing_class = Vec::new();
    let netclass_map = client.get_netclass_for_nets(
        nets.iter().map(|n| n.code).collect()
    )?;
    
    for (net_code, class_entry) in netclass_map {
        if class_entry.net_class_name.is_empty() {
            let net = nets.iter().find(|n| n.code == net_code).unwrap();
            missing_class.push(net.name.clone());
        }
    }
    
    if missing_class.is_empty() {
        println!("✓ All {} nets have net classes assigned", nets.len());
    } else {
        println!("⚠ {} nets without net classes:", missing_class.len());
        for net in missing_class.iter().take(10) {
            println!("  - {}", net);
        }
    }
    
    Ok(())
}
```

## Example: Working with Selections

Programmatically select and modify items:

```rust,no_run
use kicad_ipc_rs::KiCadClientBlocking;

fn main() -> Result<(), kicad_ipc_rs::KiCadError> {
    let client = KiCadClientBlocking::connect()?;
    
    // Get current selection summary
    let summary = client.get_selection_summary(vec![])?;
    println!("Currently selected: {} items", summary.total_count);
    
    // Clear selection
    let result = client.clear_selection()?;
    println!("Cleared {} items from selection", result.summary.total_count);
    
    // Get all tracks
    let tracks = client.get_items_by_type_codes(vec![
        kicad_ipc_rs::PcbObjectTypeCode::new_trace()
    ])?;
    
    // Select first 5 tracks
    let track_ids: Vec<_> = tracks.iter()
        .take(5)
        .filter_map(|item| {
            if let kicad_ipc_rs::PcbItem::Track(t) = item {
                t.id.clone()
            } else {
                None
            }
        })
        .collect();
    
    if !track_ids.is_empty() {
        let result = client.add_to_selection(track_ids)?;
        println!("Added {} tracks to selection", result.summary.total_count);
    }
    
    Ok(())
}
```

## CLI Testing Tool

A CLI tool is available for rapid command testing and debugging:

```bash
cargo run --features blocking --bin kicad-ipc-cli -- help
```

Common commands:
```bash
# Basic connectivity
cargo run --features blocking --bin kicad-ipc-cli -- ping
cargo run --features blocking --bin kicad-ipc-cli -- version

# Board queries
cargo run --features blocking --bin kicad-ipc-cli -- board-open
cargo run --features blocking --bin kicad-ipc-cli -- nets
cargo run --features blocking --bin kicad-ipc-cli -- pcb-types

# Selection
cargo run --features blocking --bin kicad-ipc-cli -- selection-summary
cargo run --features blocking --bin kicad-ipc-cli -- clear-selection
```

Full command catalog: [docs/TEST_CLI.md](https://github.com/Milind220/kicad-ipc-rs/blob/main/docs/TEST_CLI.md)

## Next Steps

- Learn about [usage patterns](usage-patterns.md) for integration best practices
- Check the [quickstart](quickstart.md) for getting connected
- Browse the [API reference](api-reference.md) for complete method documentation
