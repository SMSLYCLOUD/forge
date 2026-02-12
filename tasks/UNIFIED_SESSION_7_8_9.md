# FORGE IDE — UNIFIED SESSION 7+8+9 — ANTI-CRASH SEQUENTIAL BUILD
# ONE JULES AGENT executes all 3 phases in order.
# Each phase has a VERIFICATION GATE. Do NOT proceed to the next phase until the gate passes.
# PREREQUISITE: `cargo check --workspace` exits 0 BEFORE starting.

You are working on **Forge**, a GPU-accelerated code editor in Rust. You will execute 3 phases sequentially, each adding features and ending with a `cargo check` gate. Do NOT skip ahead.

---

## ⚠ ANTI-CRASH RULES (READ FIRST, VIOLATE = TASK FAILURE)

1. **READ BEFORE WRITE.** Before modifying ANY file, read it first with your file-reading tool. Never assume file contents.
2. **No `.unwrap()` in new code.** Use `if let`, `.unwrap_or_default()`, `?`, or `.unwrap_or(fallback)`.
3. **No duplicate imports.** Before adding a `use` statement, check if it already exists in the file.
4. **No phantom modules.** Before adding `mod foo;` to `main.rs`, verify `foo.rs` exists on disk.
5. **No phantom types.** Before using `SomeType::SomeVariant`, read the file that defines `SomeType` and confirm the variant exists.
6. **No phantom methods.** Before calling `obj.some_method()`, read the impl block and confirm the method exists with that exact signature.
7. **Borrow checker safety.** Never hold `&state.tab_manager.active_editor()` while also calling `active_editor_mut()`. Use scoped borrows.
8. **`cargo check --workspace`** after EVERY phase. If it fails, FIX before moving on. Never proceed with errors.
9. **`cargo fmt`** after every phase. Never submit unformatted code.
10. **If ANY API differs from what this prompt documents, trust the ACTUAL FILE, not this prompt.**

---

## CODEBASE SNAPSHOT (verified — current state)

### `crates/forge-app/src/application.rs` (1162 lines)

**`Application` struct (line 112):**
```rust
pub struct Application {
    file_path: Option<String>,
    state: Option<AppState>,
    modifiers: ModifiersState,
    current_mode: UiMode,
    config: forge_config::ForgeConfig,
    theme: forge_theme::Theme,
    // Lines 119-133: ALL commented out — find_bar, command_palette, bottom_panel,
    //   notifications, file_picker, context_menu, terminal, git_panel, split_layout, zen_mode
}
```

**`AppState` struct (line 137):**
```rust
struct AppState {
    window: Arc<Window>,
    gpu: GpuContext,
    font_system: FontSystem,
    swash_cache: SwashCache,
    cache: Cache,
    text_atlas: TextAtlas,
    viewport: Viewport,
    text_renderer: TextRenderer,
    editor_buffer: GlyphonBuffer,     // Text buffer for editor content
    gutter_buffer: GlyphonBuffer,     // Text buffer for line numbers
    tab_buffer: GlyphonBuffer,        // Text buffer for tab bar
    status_buffer: GlyphonBuffer,     // Text buffer for status bar
    breadcrumb_buffer: GlyphonBuffer, // Text buffer for breadcrumbs
    sidebar_buffer: GlyphonBuffer,    // Text buffer for file explorer sidebar
    editor: Editor,                   // ← SINGLE EDITOR — Phase 1 replaces this
    rect_renderer: RectRenderer,
    layout: LayoutZones,
    tab_bar: TabBar,                  // Visual-only tab bar — NOT TabManager
    activity_bar: ActivityBar,
    gutter: Gutter,
    status_bar_state: StatusBar,
    cursor_renderer: CursorRenderer,
    breadcrumb_bar: BreadcrumbBar,
    scrollbar: Scrollbar,
    sidebar_open: bool,
    ai_panel_open: bool,
    last_mouse_position: Option<(f32, f32)>,
    frame_timer: FrameTimer,
    render_batch: RenderBatch,
    organism_state: SharedOrganismState,
}
```

**`init_state()` (line 228):** Creates single `Editor`, opens file, sets window title, inits all glyphon buffers, creates layout.

**`render()` (line 376):** Renders rectangles → tab bar → activity bar → sidebar → gutter → editor text → cursor → scrollbar → status bar → breadcrumbs → GPU submit.

**Keyboard handler (in `window_event` match):** Lines ~700-1000. Handles Ctrl+M (mode cycle), Ctrl+B (sidebar toggle), arrow keys, Page Up/Down, Home/End, character input, backspace, Enter, Tab. All operate directly on `state.editor.*`.

### `crates/forge-app/src/tab_manager.rs` (85 lines) — VERIFIED API:
```rust
pub struct TabManager { pub tabs: Vec<Tab>, pub active: usize }
pub struct Tab { pub title: String, pub path: Option<PathBuf>, pub editor: Editor, pub is_modified: bool }
impl TabManager {
    pub fn new() -> Self
    pub fn open_file(&mut self, path: &str) -> Result<()>
    pub fn close_tab(&mut self, idx: usize)
    pub fn close_current(&mut self)
    pub fn next_tab(&mut self)
    pub fn prev_tab(&mut self)
    pub fn active_editor(&self) -> Option<&Editor>
    pub fn active_editor_mut(&mut self) -> Option<&mut Editor>
    pub fn tab_count(&self) -> usize
}
```

### `crates/forge-terminal/src/lib.rs` (107 lines) — VERIFIED API:
```rust
pub struct Terminal { pub pty: Pty, pub parser: AnsiParser, pub grid: TerminalGrid }
impl Terminal {
    pub fn new() -> Result<Self>             // Spawns shell PTY (80x24)
    pub fn resize(&mut self, cols: u16, rows: u16) -> Result<()>
    pub fn send_input(&mut self, text: &str) -> Result<()>
    pub fn tick(&mut self) -> Vec<TermEvent>  // NON-BLOCKING. Reads PTY, parses ANSI, updates grid.
    pub fn render_grid(&self) -> &TerminalGrid
}
// ⚠ There is NO poll_output() and NO is_alive(). Use tick().
```

### `crates/forge-terminal/src/grid.rs` — READ THIS FILE to find TerminalGrid fields (cursor_row, cursor_col, rows, cols, cells).

### `crates/forge-search/src/lib.rs` — exports:
```rust
pub use content::{ContentSearcher, SearchOpts, SearchResult};
pub use fuzzy::{fuzzy_filter, fuzzy_score};
```

### Other verified modules in `crates/forge-app/src/` (all exist, all have `new()` or `default()`):
- `file_tree_ui.rs` — `FileTreeUi { visible, scroll_offset, selected_index, hovered_index }`, `DisplayNode { label, depth, is_dir, expanded }`, `render_rects(&self, nodes, zone) -> Vec<Rect>`
- `minimap.rs` — `Minimap::build(total_lines, viewport_start, viewport_end) -> Self`, `click_to_line(y_fraction) -> usize`
- `code_folding.rs` — `FoldingManager::new()`, `compute_ranges(&mut self, text) -> Vec<FoldRange>`, `toggle_fold(line)`, `is_line_visible(line) -> bool`
- `indent_guides.rs` — `IndentGuides::compute(text, tab_size, cursor_line) -> Vec<GuideLine>`, `GuideLine { line, column, depth, is_active, scope_start, scope_end }`
- `bracket_match.rs` — READ THIS FILE to find the actual function signature
- `find_bar.rs` — READ THIS FILE for `FindBar` API
- `command_palette.rs` — READ THIS FILE for `CommandPalette` API
- `bottom_panel.rs` — READ THIS FILE for `BottomPanel` API
- `notifications.rs` — READ THIS FILE for `NotificationManager` API
- `context_menu.rs` — READ THIS FILE for `ContextMenu` API
- `settings_ui.rs` — READ THIS FILE for `SettingsUi` API
- `comment_toggle.rs` — READ THIS FILE for `toggle_comment` signature
- `git_gutter.rs` — READ THIS FILE for gutter mark API
- `outline_panel.rs` — READ THIS FILE for `OutlinePanel` API

### `crates/forge-app/Cargo.toml` — currently has:
```toml
forge-core, forge-renderer, forge-confidence, forge-config, forge-theme, forge-syntax,
forge-debug, forge-lsp, forge-plugin, forge-terminal, forge-search, git2, tree-sitter,
wgpu, winit, glyphon, pollster, arboard, anyhow, tracing, tracing-subscriber, serde, toml, dirs-next, regex
```

### `crates/forge-theme/src/lib.rs` — READ THIS FILE to find ThemeColors fields.

### `rect_renderer::Rect` struct:
```rust
pub struct Rect { pub x: f32, pub y: f32, pub width: f32, pub height: f32, pub color: [f32; 4] }
```

### Key constants from `ui.rs`:
```rust
pub struct LayoutConstants;
impl LayoutConstants {
    pub const FONT_SIZE: f32 = 14.0;
    pub const LINE_HEIGHT: f32 = 22.0;
    pub const SMALL_FONT_SIZE: f32 = 12.0;
    // ... more constants
}
```

---

# ═══════════════════════════════════════════
# PHASE 1: MULTI-FILE EDITING + FILE EXPLORER
# ═══════════════════════════════════════════

## STEP 1.1: Replace `editor: Editor` with `tab_manager: TabManager` in AppState

**In `application.rs`:**

1. Add import at top: `use crate::tab_manager::TabManager;`
2. In `AppState` struct, REPLACE `editor: Editor,` (line 159) WITH `tab_manager: TabManager,`
3. Add a `bottom_panel_buffer: GlyphonBuffer,` field to AppState (for future terminal text)

## STEP 1.2: Update `init_state()` to use TabManager

Replace the editor initialization block (lines 238-252) with:
```rust
let mut tab_manager = TabManager::new();
if let Some(ref path) = self.file_path {
    if let Err(e) = tab_manager.open_file(path) {
        tracing::warn!("Failed to open {}: {}", path, e);
    }
}

let window_title = tab_manager.active_editor()
    .map(|e| e.window_title())
    .unwrap_or_else(|| "Forge — [no file]".to_string());
window.set_title(&window_title);
```

Add bottom_panel_buffer creation after sidebar_buffer:
```rust
let bottom_panel_buffer = GlyphonBuffer::new(
    &mut font_system,
    Metrics::new(LayoutConstants::SMALL_FONT_SIZE, LayoutConstants::LINE_HEIGHT),
);
```

In the `self.state = Some(AppState { ... })` block, replace `editor,` with `tab_manager,` and add `bottom_panel_buffer,`.

## STEP 1.3: Find-and-replace ALL `state.editor` references

This is the hardest part. Search for EVERY `state.editor` in application.rs. There are ~30-40 occurrences. Replace each one:

**For read access (rendering, scroll position, line counts):**
```rust
// Wrap the entire render section that uses editor in:
if let Some(editor) = state.tab_manager.active_editor() {
    // Use `editor.` instead of `state.editor.`
    let scroll_top = editor.scroll_top();
    let total_lines = editor.total_lines();
    // ... all rendering code ...
} else {
    // No file open — skip editor rendering
}
```

**For write access (character input, movement):**
```rust
// OLD: state.editor.insert_char(ch);
// NEW:
if let Some(ed) = state.tab_manager.active_editor_mut() {
    ed.insert_char(ch);
}
```

**CRITICAL BORROW CHECKER RULE:** You CANNOT do this:
```rust
let editor = state.tab_manager.active_editor(); // borrows state
state.tab_manager.active_editor_mut(); // ERROR: already borrowed
```
Use SCOPED borrows — read what you need, drop the borrow, then mutate:
```rust
let scroll_top = state.tab_manager.active_editor().map(|e| e.scroll_top()).unwrap_or(0);
// borrow dropped here
if let Some(ed) = state.tab_manager.active_editor_mut() {
    ed.scroll_to(scroll_top + 1);
}
```

**Common replacements (do ALL of these):**

| Original | Replacement |
|----------|-------------|
| `state.editor.scroll_top()` | `state.tab_manager.active_editor().map(\|e\| e.scroll_top()).unwrap_or(0)` |
| `state.editor.total_lines()` | `state.tab_manager.active_editor().map(\|e\| e.total_lines()).unwrap_or(1)` |
| `state.editor.cursor_line()` | `state.tab_manager.active_editor().map(\|e\| e.cursor_line()).unwrap_or(0)` |
| `state.editor.cursor_col()` | `state.tab_manager.active_editor().map(\|e\| e.cursor_col()).unwrap_or(0)` |
| `state.editor.insert_char(ch)` | `if let Some(ed) = state.tab_manager.active_editor_mut() { ed.insert_char(ch); }` |
| `state.editor.backspace()` | `if let Some(ed) = state.tab_manager.active_editor_mut() { ed.backspace(); }` |
| `state.editor.move_left()` | `if let Some(ed) = state.tab_manager.active_editor_mut() { ed.move_left(); }` |
| `state.editor.move_right()` | `if let Some(ed) = state.tab_manager.active_editor_mut() { ed.move_right(); }` |
| `state.editor.move_up()` | `if let Some(ed) = state.tab_manager.active_editor_mut() { ed.move_up(); }` |
| `state.editor.move_down()` | `if let Some(ed) = state.tab_manager.active_editor_mut() { ed.move_down(); }` |
| `state.editor.new_line()` | `if let Some(ed) = state.tab_manager.active_editor_mut() { ed.new_line(); }` |
| `state.editor.insert_tab()` | `if let Some(ed) = state.tab_manager.active_editor_mut() { ed.insert_tab(); }` |
| `state.editor.window_title()` | `state.tab_manager.active_editor().map(\|e\| e.window_title()).unwrap_or_else(\|\| "Forge".into())` |
| `state.editor.buffer.text()` | `state.tab_manager.active_editor().map(\|e\| e.buffer.text()).unwrap_or_default()` |
| `state.editor.buffer.rope()` | Read-only: extract inside `if let Some(editor) = state.tab_manager.active_editor()` block |
| `state.editor.rehighlight()` | `if let Some(ed) = state.tab_manager.active_editor_mut() { ed.rehighlight(); }` |

**The render loop pattern:**
In the `render()` function, wrap the editor text rendering section:
```rust
// Get editor data with a short-lived borrow
let (editor_text, scroll_top, total_lines, cursor_line, cursor_col) = {
    if let Some(editor) = state.tab_manager.active_editor() {
        let text = editor.visible_text(/* args from current code */);
        (text, editor.scroll_top(), editor.total_lines(), editor.cursor_line(), editor.cursor_col())
    } else {
        ("Welcome to Forge Editor\n\nOpen a file to start editing.".to_string(), 0, 1, 0, 0)
    }
};
// Now use those extracted values — the borrow is dropped
```

READ the actual render() function to see exactly how `state.editor` is used, then adapt.

## STEP 1.4: Tab Switching Shortcuts

Add to keyboard handler:
```rust
// Ctrl+Tab → next tab
Key::Named(NamedKey::Tab) if self.modifiers.control_key() => {
    state.tab_manager.next_tab();
    state.window.request_redraw();
}
// Ctrl+W → close current tab
Key::Character(ref c) if c == "w" && self.modifiers.control_key() => {
    state.tab_manager.close_current();
    state.window.request_redraw();
}
```

Update tab bar text rendering to show `tab_manager.tabs`:
```rust
let tab_text = if state.tab_manager.tabs.is_empty() {
    "  Welcome".to_string()
} else {
    state.tab_manager.tabs.iter().enumerate()
        .map(|(i, tab)| {
            let modified = if tab.is_modified { "● " } else { "" };
            if i == state.tab_manager.active {
                format!(" {}{} ", modified, tab.title)
            } else {
                format!("   {}{}  ", modified, tab.title)
            }
        })
        .collect::<Vec<_>>()
        .join("│")
};
```

## STEP 1.5: Real File Explorer

**Create `crates/forge-app/src/file_explorer.rs`:**

```rust
use crate::file_tree_ui::DisplayNode;
use std::path::{Path, PathBuf};

pub struct FileExplorer {
    pub root: Option<PathBuf>,
    pub nodes: Vec<DisplayNode>,
    pub paths: Vec<PathBuf>,
    pub selected: Option<usize>,
}

impl FileExplorer {
    pub fn new() -> Self {
        Self { root: None, nodes: Vec::new(), paths: Vec::new(), selected: None }
    }

    pub fn scan_directory(&mut self, root: &Path) -> anyhow::Result<()> {
        self.root = Some(root.to_path_buf());
        self.nodes.clear();
        self.paths.clear();
        self.scan_recursive(root, 0)?;
        Ok(())
    }

    fn scan_recursive(&mut self, dir: &Path, depth: usize) -> anyhow::Result<()> {
        if depth > 3 { return Ok(()); } // Limit depth
        let mut entries: Vec<_> = std::fs::read_dir(dir)?
            .filter_map(|e| e.ok())
            .collect();
        entries.sort_by(|a, b| {
            let a_dir = a.file_type().map(|t| t.is_dir()).unwrap_or(false);
            let b_dir = b.file_type().map(|t| t.is_dir()).unwrap_or(false);
            b_dir.cmp(&a_dir).then(a.file_name().cmp(&b.file_name()))
        });
        for entry in entries {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') || name == "target" || name == "node_modules" { continue; }
            let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
            let path = entry.path();
            self.nodes.push(DisplayNode { label: name, depth, is_dir, expanded: depth == 0 && is_dir });
            self.paths.push(path.clone());
            if is_dir && depth == 0 {
                let _ = self.scan_recursive(&path, depth + 1);
            }
        }
        Ok(())
    }

    pub fn get_path(&self, index: usize) -> Option<&Path> {
        self.paths.get(index).map(|p| p.as_path())
    }

    pub fn toggle_expand(&mut self, _index: usize) {
        if let Some(node) = self.nodes.get_mut(_index) {
            if node.is_dir { node.expanded = !node.expanded; }
        }
    }
}

impl Default for FileExplorer {
    fn default() -> Self { Self::new() }
}
```

Add `pub mod file_explorer;` to `main.rs` (after the Session 3 block, line 83).

## STEP 1.6: Wire File Explorer to AppState

Add to `AppState`:
```rust
file_explorer: crate::file_explorer::FileExplorer,
```

In `init_state()`, after organism setup:
```rust
let mut file_explorer = crate::file_explorer::FileExplorer::new();
let cwd = std::env::current_dir().unwrap_or_default();
let _ = file_explorer.scan_directory(&cwd);
```

Add `file_explorer,` to the `AppState { ... }` initializer.

Replace the hardcoded sidebar text with dynamic file tree:
```rust
// In the sidebar rendering section, replace the hardcoded text with:
let sidebar_text = if state.sidebar_open {
    let mut text = "  EXPLORER\n\n".to_string();
    for node in &state.file_explorer.nodes {
        let indent = "  ".repeat(node.depth + 1);
        let icon = if node.is_dir {
            if node.expanded { "📂 " } else { "📁 " }
        } else { "📄 " };
        text.push_str(&format!("{}{}{}\n", indent, icon, node.label));
    }
    text
} else {
    String::new()
};
```

## STEP 1.7: Ctrl+S Save

In keyboard handler:
```rust
Key::Character(ref c) if c == "s" && self.modifiers.control_key() => {
    if let Some(tab) = state.tab_manager.tabs.get(state.tab_manager.active) {
        if let Some(ref path) = tab.path {
            if let Some(ed) = state.tab_manager.active_editor() {
                let text = ed.buffer.text();
                if let Err(e) = std::fs::write(path, &text) {
                    tracing::error!("Save failed: {}", e);
                } else {
                    tracing::info!("Saved: {}", path.display());
                }
            }
        }
    }
    state.window.request_redraw();
}
```

## ✅ GATE 1: Run `cargo check --workspace` and `cargo fmt`. MUST exit 0. Fix ALL errors before proceeding.

---

# ═══════════════════════════════════════
# PHASE 2: TERMINAL + FIND BAR + BRACKET MATCH
# ═══════════════════════════════════════

## STEP 2.1: Wire Terminal to Bottom Panel

Add to `AppState`:
```rust
terminal: Option<forge_terminal::Terminal>,
bottom_panel_focused: bool,
```

Initialize: `terminal: None, bottom_panel_focused: false,`

**Read `bottom_panel.rs` FIRST** to understand BottomPanel struct.

Uncomment and wire `bottom_panel` in Application struct:
```rust
// In Application struct, UNCOMMENT:
bottom_panel: crate::bottom_panel::BottomPanel,
// Initialize in Application::new():
bottom_panel: crate::bottom_panel::BottomPanel::default(),
```

Add `Ctrl+Backtick` to toggle terminal:
```rust
Key::Character(ref c) if c == "`" && self.modifiers.control_key() => {
    self.bottom_panel.visible = !self.bottom_panel.visible;
    if self.bottom_panel.visible && state.terminal.is_none() {
        match forge_terminal::Terminal::new() {
            Ok(term) => state.terminal = Some(term),
            Err(e) => tracing::warn!("Terminal failed: {}", e),
        }
    }
    // Recompute layout to include/exclude bottom panel
    let (w, h) = state.gpu.size();
    state.layout = LayoutZones::compute(w as f32, h as f32, state.sidebar_open, self.bottom_panel.visible);
    state.window.request_redraw();
}
```

**READ `ui.rs`** to verify that `LayoutZones::compute` accepts a bottom_panel_visible parameter. If it doesn't, add one or adapt.

In render loop, after all other rendering, add terminal rendering:
```rust
if self.bottom_panel.visible {
    if let Some(ref bp) = state.layout.bottom_panel {
        // Background
        state.render_batch.push(crate::rect_renderer::Rect {
            x: bp.x, y: bp.y, width: bp.width, height: bp.height,
            color: [0.05, 0.05, 0.08, 1.0],
        });

        // Terminal text
        if let Some(ref mut term) = state.terminal {
            let _events = term.tick();
            let grid = term.render_grid();
            let mut term_text = String::new();
            for row in 0..grid.rows {
                for col in 0..grid.cols {
                    // READ grid.rs to find how cells are accessed
                    // It might be grid.cells[row][col].ch or grid.cell(row, col)
                    // Adapt based on actual API
                }
                term_text.push('\n');
            }
            state.bottom_panel_buffer.set_size(
                &mut state.font_system,
                Some(bp.width), Some(bp.height),
            );
            state.bottom_panel_buffer.set_text(
                &mut state.font_system,
                &term_text,
                Attrs::new().family(Family::Monospace)
                    .color(GlyphonColor::rgb(229, 229, 229)),
                Shaping::Advanced,
            );
        }
    }
}
```

**Forward keyboard input to terminal when bottom panel is focused:**
```rust
// At the START of keyboard handling, before editor input:
if state.bottom_panel_focused {
    if let Some(ref mut term) = state.terminal {
        match &key {
            Key::Named(NamedKey::Enter) => { let _ = term.send_input("\r\n"); }
            Key::Named(NamedKey::Backspace) => { let _ = term.send_input("\x7f"); }
            Key::Character(ref c) => { let _ = term.send_input(c); }
            _ => {}
        }
        state.window.request_redraw();
        return; // Don't process as editor input
    }
}
```

## STEP 2.2: Wire Find Bar (Ctrl+F)

**READ `find_bar.rs` FIRST.** Then uncomment in Application struct and wire:
```rust
// In Application struct:
find_bar: crate::find_bar::FindBar,
// In Application::new():
find_bar: crate::find_bar::FindBar::default(),
```

Add keyboard shortcut:
```rust
Key::Character(ref c) if c == "f" && self.modifiers.control_key() => {
    self.find_bar.toggle();
    state.window.request_redraw();
}
```

Render find bar overlay when visible (READ find_bar.rs for render_rects or similar method):
```rust
if self.find_bar.visible {
    // Render find bar at top-right of editor area
    let fb_width = 400.0;
    let fb_height = 36.0;
    let fb_x = state.layout.editor.x + state.layout.editor.width - fb_width - 20.0;
    let fb_y = state.layout.editor.y + 4.0;
    state.render_batch.push(crate::rect_renderer::Rect {
        x: fb_x, y: fb_y, width: fb_width, height: fb_height,
        color: [0.18, 0.20, 0.26, 0.98],
    });
}
```

## STEP 2.3: Wire Command Palette (Ctrl+Shift+P)

**READ `command_palette.rs` FIRST.** Uncomment and wire similarly to find bar.

```rust
Key::Character(ref c) if c == "P" && self.modifiers.control_key() && self.modifiers.shift_key() => {
    self.command_palette.toggle();
    state.window.request_redraw();
}
```

Render as centered overlay:
```rust
if self.command_palette.visible {
    let cp_width = 500.0;
    let cp_height = 300.0;
    let (w, h) = state.gpu.size();
    let cp_x = (w as f32 - cp_width) / 2.0;
    let cp_y = 80.0;
    state.render_batch.push(crate::rect_renderer::Rect {
        x: cp_x, y: cp_y, width: cp_width, height: cp_height,
        color: [0.14, 0.15, 0.20, 0.98],
    });
}
```

## STEP 2.4: Bracket Matching

**READ `bracket_match.rs` FIRST** to find the actual function signature.

In render loop, after editor text rendering:
```rust
if let Some(editor) = state.tab_manager.active_editor() {
    let text = editor.buffer.text();
    let cursor_line = editor.cursor_line();
    let cursor_col = editor.cursor_col();
    let scroll_top = editor.scroll_top();
    // Convert cursor to byte offset — READ editor.rs to find the right method
    // Likely: editor.buffer.rope().line_to_byte(cursor_line) + cursor_col
    let line_start = editor.buffer.rope().line_to_byte(cursor_line);
    let cursor_byte = line_start + cursor_col;

    // Call bracket_match — use the ACTUAL function signature from the file
    // It might be: find_matching_bracket(&text, cursor_byte)
    // Or: BracketMatcher::find_match(&text, cursor_byte)
    // READ THE FILE FIRST
}
```

## ✅ GATE 2: Run `cargo check --workspace` and `cargo fmt`. MUST exit 0. Fix ALL errors before proceeding.

---

# ═══════════════════════════════════════
# PHASE 3: THEME COLORS + SETTINGS + POLISH
# ═══════════════════════════════════════

## STEP 3.1: Theme-Aware Rendering

**READ `crates/forge-theme/src/lib.rs` FIRST** to find exact `ThemeColors` fields.

Add helper methods to Application:
```rust
impl Application {
    fn theme_color(rgb: [u8; 3]) -> [f32; 4] {
        [rgb[0] as f32 / 255.0, rgb[1] as f32 / 255.0, rgb[2] as f32 / 255.0, 1.0]
    }
    fn theme_glyph_color(rgb: [u8; 3]) -> GlyphonColor {
        GlyphonColor::rgb(rgb[0], rgb[1], rgb[2])
    }
}
```

Then search for ALL hardcoded color values in application.rs (like `[0.12, 0.12, 0.18, 1.0]`, `GlyphonColor::rgb(248, 248, 242)`) and replace with theme lookups. Match theme field names to what ACTUALLY exists in ThemeColors.

## STEP 3.2: Settings UI (Ctrl+,)

**READ `settings_ui.rs` FIRST.** Uncomment in Application struct if not already.

```rust
Key::Character(ref c) if c == "," && self.modifiers.control_key() => {
    // READ settings_ui.rs to find toggle method
    state.window.request_redraw();
}
```

## STEP 3.3: Notifications

**READ `notifications.rs` FIRST.** Uncomment and wire. Render as toast popups bottom-right.

## STEP 3.4: Context Menu (Right-Click)

**READ `context_menu.rs` FIRST.** Wire right-click to show menu overlay.

## STEP 3.5: Update Status Bar

Update status bar text to show:
```rust
let language = state.tab_manager.tabs.get(state.tab_manager.active)
    .and_then(|t| t.path.as_ref())
    .map(|p| {
        let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
        match ext {
            "rs" => "Rust", "js" | "jsx" => "JavaScript", "ts" | "tsx" => "TypeScript",
            "py" => "Python", "json" => "JSON", "toml" => "TOML", "md" => "Markdown",
            "html" => "HTML", "css" => "CSS", _ => "Plain Text",
        }
    })
    .unwrap_or("Plain Text");

let cursor_line = state.tab_manager.active_editor().map(|e| e.cursor_line() + 1).unwrap_or(1);
let cursor_col = state.tab_manager.active_editor().map(|e| e.cursor_col() + 1).unwrap_or(1);
let fps = if state.frame_timer.avg_frame_time_ms > 0.0 {
    (1000.0 / state.frame_timer.avg_frame_time_ms) as u32
} else { 0 };

let status_text = format!(
    "  Forge IDE  │  Ln {}, Col {}  │  UTF-8  │  {}  │  {} fps  │  {} tabs",
    cursor_line, cursor_col, language, fps, state.tab_manager.tab_count()
);
```

## STEP 3.6: Welcome Screen

When no file is open:
```rust
if state.tab_manager.tabs.is_empty() {
    let welcome_text = "\n\n\n\
        \t\t\t🔥 FORGE EDITOR\n\n\
        \t\t\tGPU-Accelerated Code Editing\n\n\
        \t\t\tCtrl+O    Open File\n\
        \t\t\tCtrl+P    Quick Open\n\
        \t\t\tCtrl+`    Toggle Terminal\n\
        \t\t\tCtrl+,    Settings\n";
    // Set this text on editor_buffer
}
```

## ✅ GATE 3 (FINAL): Run ALL of these:
```bash
cargo fmt
cargo check --workspace
cargo clippy --workspace -- -W clippy::all
cargo test --workspace
```
ALL must exit 0 (clippy warnings OK if pre-existing).

---

# FINAL CHECKLIST — Verify before submitting:

- [ ] `state.editor` does NOT appear ANYWHERE in application.rs (replaced with tab_manager)
- [ ] `file_explorer.rs` exists and `pub mod file_explorer;` is in main.rs
- [ ] Ctrl+Tab / Ctrl+W work for tab switching
- [ ] Ctrl+S saves current file
- [ ] Ctrl+F toggles find bar
- [ ] Ctrl+Shift+P toggles command palette
- [ ] Ctrl+` toggles terminal panel
- [ ] Ctrl+, toggles settings
- [ ] No `.unwrap()` in new code
- [ ] No duplicate imports
- [ ] `cargo check --workspace` = 0 errors
- [ ] `cargo fmt --check` = 0 changes needed
