use anyhow::Result;
use forge_core::Buffer;
use tracing::info;
use tracing_subscriber;
use forge_confidence::engine::ConfidenceEngine;
use forge_confidence::db::ConfidenceDb;
use forge_propagation::graph::GraphStore;
use std::path::Path;

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("🔥 Forge Editor - Phase 1");

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();

    // Load buffer
    let buffer = if args.len() > 1 {
        info!("Loading file: {}", args[1]);
        Buffer::from_file(&args[1])?
    } else {
        info!("Creating empty buffer");
        Buffer::new()
    };

    info!(
        "Buffer loaded: {} lines, {} bytes",
        buffer.len_lines(),
        buffer.len_bytes()
    );

    // Initialize engines
    let _confidence_engine = ConfidenceEngine;
    let _graph = GraphStore::new();
    let _db = ConfidenceDb::open(Path::new("confidence.db")); // In-memory or local file

    info!("Sub-Binary IDE engines initialized");

    // Phase 1: We just verify the buffer loads correctly
    // Future phases will add window, renderer, event loop

    println!("\n✨ Forge Phase 1 - Core Engine ✨");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!(
        "Buffer: {} lines, {} bytes",
        buffer.len_lines(),
        buffer.len_bytes()
    );
    println!("Selection: {:?}", buffer.selection());
    println!("Dirty: {}", buffer.is_dirty());

    if let Some(path) = buffer.path() {
        println!("Path: {}", path);
    }

    println!("\n🎯 Core features implemented:");
    println!("  ✓ Rope-based text buffer");
    println!("  ✓ Transaction system (atomic edits)");
    println!("  ✓ History tree (non-linear undo/redo)");
    println!("  ✓ Multiple selections/cursors");
    println!("  ✓ File I/O with encoding detection");

    println!("\n📦 Next: Phase 2 will add GPU rendering, window, and basic editor UI");

    Ok(())
}
