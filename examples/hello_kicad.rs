//! Minimal "hello world" example — connect to KiCad and print version info.
//!
//! Run with:
//!     cargo run --example hello_kicad --features blocking

#[cfg(feature = "blocking")]
use kicad_ipc_rs::KiCadClientBlocking;

#[cfg(feature = "blocking")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to a running KiCad instance.
    // Auto-detects the IPC socket; override with KICAD_API_SOCKET env var.
    let client = KiCadClientBlocking::connect()?;

    // Health check — verifies the connection is alive.
    client.ping()?;
    println!("✓ Connected to KiCad");

    // Retrieve version metadata.
    let version = client.get_version()?;
    println!("  Version : {}", version.full_version);
    println!(
        "  SemVer  : {}.{}.{}",
        version.major, version.minor, version.patch
    );

    // Check whether a PCB document is open.
    if client.has_open_board()? {
        let path = client.get_current_project_path()?;
        println!("  Project : {}", path.display());
    } else {
        println!("  (no board open)");
    }

    Ok(())
}

#[cfg(not(feature = "blocking"))]
fn main() {
    eprintln!("This example requires the blocking feature:");
    eprintln!("  cargo run --example hello_kicad --features blocking");
    std::process::exit(1);
}
