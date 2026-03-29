//! Inspect the current PCB board — list nets, layers, and origin.
//!
//! Run with:
//!     cargo run --example board_inspector --features blocking

#[cfg(feature = "blocking")]
use kicad_ipc_rs::{BoardOriginKind, KiCadClientBlocking};

#[cfg(feature = "blocking")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = KiCadClientBlocking::connect()?;
    client.ping()?;

    if !client.has_open_board()? {
        eprintln!("No board is open in KiCad. Open a .kicad_pcb file first.");
        std::process::exit(1);
    }

    // ── Nets ──────────────────────────────────────────────
    let nets = client.get_nets()?;
    println!("Nets ({} total):", nets.len());
    for net in nets.iter().take(20) {
        println!("  [{:>3}] {}", net.code, net.name);
    }
    if nets.len() > 20 {
        println!("  … and {} more", nets.len() - 20);
    }

    // ── Enabled layers ────────────────────────────────────
    let layers = client.get_board_enabled_layers()?;
    println!(
        "
Enabled layers ({} copper, {} total IDs):",
        layers.copper_layer_count,
        layers.layers.len()
    );
    for layer in layers.layers.iter().take(10) {
        println!("  layer {:>2} → {}", layer.id, layer.name);
    }
    if layers.layers.len() > 10 {
        println!("  … and {} more", layers.layers.len() - 10);
    }

    // ── Board origins ─────────────────────────────────────
    let grid_origin = client.get_board_origin(BoardOriginKind::Grid)?;
    let drill_origin = client.get_board_origin(BoardOriginKind::Drill)?;
    println!(
        "
Grid origin  : ({}, {}) nm",
        grid_origin.x_nm, grid_origin.y_nm
    );
    println!(
        "Drill origin : ({}, {}) nm",
        drill_origin.x_nm, drill_origin.y_nm
    );

    // ── Active layer ──────────────────────────────────────
    let active = client.get_active_layer()?;
    println!(
        "
Active layer : {} ({})",
        active.id, active.name
    );

    Ok(())
}

#[cfg(not(feature = "blocking"))]
fn main() {
    eprintln!("This example requires the blocking feature:");
    eprintln!("  cargo run --example board_inspector --features blocking");
    std::process::exit(1);
}
