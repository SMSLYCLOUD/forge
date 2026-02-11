# Forge — GPU-Accelerated Code Editor

A high-performance code editor built in Rust with direct GPU rendering via wgpu.

## Features

- 🖥 GPU-rendered text with wgpu + glyphon
- 📝 Full-featured text editing (undo/redo, multi-cursor, find/replace)
- 🎨 7 built-in themes (Forge Dark, Forge Light, Monokai, Dracula, One Dark, Solarized, Nord)
- ⌨️ VS Code-compatible keybindings
- 🔍 Project-wide fuzzy search
- 📁 Multi-root workspace support
- 🖥 Integrated terminal with ANSI support
- 🌳 Git integration (status, blame, diff, branches)
- 🧩 Extension system (LSP, task runner)
- ♿ Accessibility layer with ARIA roles
- 🤖 AI integration framework (inline completions, chat)
- ⚡ Crash recovery and auto-save
- 📐 Code folding, indent guides, minimap
- 🔧 TOML-based configuration

## Building

```bash
cargo build --release
```

## Running

```bash
cargo run -p forge-app
```

## Testing

```bash
cargo test --workspace
```

## Architecture

Forge is organized as a Cargo workspace with the following crates:

| Crate | Description |
|-------|-------------|
| forge-core | Rope buffer, transactions, undo/redo, file I/O |
| forge-renderer | wgpu GPU pipeline, text atlas, rect renderer |
| forge-window | winit event loop, windowing |
| forge-app | Main application, UI components |
| forge-config | TOML configuration |
| forge-theme | Color themes engine |
| forge-input | Keyboard/mouse input, clipboard |
| forge-keybindings | Keyboard shortcut system |
| forge-types | Shared types (Color, Rect, Position) |
| forge-workspace | Multi-root workspace |
| forge-terminal | PTY, ANSI parser, grid buffer |
| forge-search | Fuzzy finder, content search |
| forge-lsp | Language Server Protocol client |
| forge-surfaces | UI surface manager |
| forge-agent | AI agent (chat, inline completions) |
| forge-net | Network/HTTP client |

## License

MIT OR Apache-2.0
