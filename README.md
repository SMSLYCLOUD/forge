# Forge Editor

A next-generation code editor built from the ground up in Rust.

## Phase 1: Core Engine ✅

**Implemented:**
- ✅ `forge-core` - Rope-based text buffer with transaction system
- ✅ `forge-renderer` - GPU rendering foundation (structure only)
- ✅ `forge-window` - Platform abstraction (structure only)
- ✅ `forge-app` - Application entry point

**Features:**
- Rope data structure for efficient text manipulation
- Transaction system for atomic, invertible edits
- History tree for non-linear undo/redo
- Multiple selections/cursors
- File I/O with encoding detection

## Building

```bash
cargo build --release
```

## Running

```bash
# Open a file
cargo run --release -- path/to/file.txt

# Or start with empty buffer
cargo run --release
```

## Testing

```bash
cargo test
```

## Architecture

```
forge/
├── crates/
│   ├── forge-core/      # Text buffer engine
│   ├── forge-renderer/  # GPU text rendering
│   ├── forge-window/    # OS windowing
│   └── forge-app/       # Main application
└── Cargo.toml          # Workspace root
```

## Next Steps (Future Phases)

- Phase 2: Tree-sitter, LSP, search, file tree
- Phase 3: Terminal + Git integration
- Phase 4: WASM plugin system
- Phase 5: CRDT collaboration
- Phase 6: AI integration

## License

MIT OR Apache-2.0
