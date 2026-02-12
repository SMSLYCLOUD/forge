# SESSION 7 — Real File Explorer + Multi-File Editing + Visual Features
# ONE JULES TASK — Copy this ENTIRE file as one Jules prompt.
# PREREQUISITE: Session 6 must be merged. `cargo check --workspace` exits 0.
# THIS SESSION MODIFIES `application.rs`.
# ═══════════════════════════════════════════════════════════════

You are working on **Forge**, a GPU-accelerated code editor written in Rust. This session replaces the hardcoded file tree with a real filesystem explorer, replaces single-file editing with multi-file tab management, and renders minimap + code folding + indent guides.

## CRITICAL CONTEXT

### Existing modules you MUST use (already in the codebase):

**`crates/forge-app/src/tab_manager.rs`** — Already exists. Key API:
```rust
pub struct TabManager { pub tabs: Vec<Tab>, pub active: usize }
pub struct Tab { pub title: String, pub path: Option<PathBuf>, pub editor: Editor, pub is_modified: bool }
impl TabManager {
    pub fn new() -> Self
    pub fn open_file(&mut self, path: &str) -> Result<()>  // Opens file, creates Editor, adds tab
    pub fn close_tab(&mut self, idx: usize)
    pub fn close_current(&mut self)
    pub fn next_tab(&mut self)
    pub fn prev_tab(&mut self)
    pub fn active_editor(&self) -> Option<&Editor>
    pub fn active_editor_mut(&mut self) -> Option<&mut Editor>
    pub fn tab_count(&self) -> usize
}
```

**`crates/forge-app/src/file_tree_ui.rs`** — Already exists. Key API:
```rust
pub struct FileTreeUi { pub scroll_offset: usize, pub selected_index: Option<usize>, pub hovered_index: Option<usize> }
pub struct DisplayNode { pub label: String, pub depth: usize, pub is_dir: bool, pub expanded: bool }
impl FileTreeUi {
    pub fn new() -> Self
    pub fn render_rects(&self, nodes: &[DisplayNode], zone: &Zone) -> Vec<Rect>
}
```

**`crates/forge-app/src/minimap.rs`** — Already exists:
```rust
pub struct Minimap { pub lines: Vec<MinimapLine>, pub viewport_start: usize, pub viewport_end: usize, pub total_lines: usize }
pub struct MinimapLine { pub line: usize, pub color: [f32; 3] }
impl Minimap {
    pub fn build(total_lines: usize, viewport_start: usize, viewport_end: usize) -> Self
    pub fn click_to_line(&self, y_fraction: f32) -> usize
}
```

**`crates/forge-app/src/code_folding.rs`** — Already exists:
```rust
pub struct FoldRange { pub start_line: usize, pub end_line: usize, pub kind: FoldKind, pub collapsed: bool }
pub struct FoldingManager { pub ranges: Vec<FoldRange> }
impl FoldingManager {
    pub fn new() -> Self
    pub fn compute_ranges(&mut self, text: &str) -> Vec<FoldRange>
    pub fn toggle_fold(&mut self, line: usize)
    pub fn is_line_visible(&self, line: usize) -> bool
    pub fn fold_all(&mut self)
    pub fn unfold_all(&mut self)
}
```

**`crates/forge-app/src/indent_guides.rs`** — Already exists:
```rust
pub struct GuideLine { pub line: usize, pub column: usize, pub depth: usize, pub is_active: bool, pub scope_start: usize, pub scope_end: usize }
impl IndentGuides {
    pub fn compute(text: &str, tab_size: u32, cursor_line: usize) -> Vec<GuideLine>
}
```

### Current `application.rs` structure:
- `Application` struct has: `file_path`, `state: Option<AppState>`, `config`, `theme`
- `AppState` has: `editor: Editor` (single editor), `tab_bar: TabBar` (visual only — NOT TabManager)
- Render loop renders editor text from `state.editor`
- Key handler operates on `state.editor`

### What MUST change:
1. Replace `state.editor` with `state.tab_manager: TabManager`
2. All code that reads/writes `state.editor` must go through `state.tab_manager.active_editor()` / `active_editor_mut()`
3. File explorer must scan real filesystem
4. Clicking a file in the sidebar opens it in a new tab

---

## RULES (MANDATORY)

1. No `.unwrap()` in production. Use `if let`, `.unwrap_or_default()`, or `?`.
2. `cargo fmt` + `cargo clippy -- -D warnings` = zero warnings.
3. Do NOT delete existing rendering pattern — adapt it.
4. **Read existing files FIRST before modifying them.**

---

## TASK 1: Replace Single Editor with TabManager

**Modify `application.rs`:**

Replace `editor: Editor` in `AppState` with `tab_manager: TabManager`:

```rust
// In AppState, REPLACE:
editor: Editor,
// WITH:
tab_manager: crate::tab_manager::TabManager,
```

In `init_state()`, replace the editor initialization:
```rust
// REPLACE the editor creation block with:
let mut tab_manager = crate::tab_manager::TabManager::new();
if let Some(ref path) = self.file_path {
    if let Err(e) = tab_manager.open_file(path) {
        tracing::warn!("Failed to open {}: {}", path, e);
    }
}
```

**Then find EVERY reference to `state.editor` in `application.rs` and replace:**

For read access (scroll, cursor position, total lines, etc.):
```rust
// OLD: state.editor.scroll_top()
// NEW: state.tab_manager.active_editor().map(|e| e.scroll_top()).unwrap_or(0)
```

For write access (insert_char, backspace, move_left, etc.):
```rust
// OLD: state.editor.insert_char(ch)
// NEW: if let Some(ed) = state.tab_manager.active_editor_mut() { ed.insert_char(ch); }
```

This is tedious but CRITICAL. Search for every `state.editor.` and replace with the TabManager indirection. There will be ~30-40 references. A few examples:

| Old | New |
|-----|-----|
| `state.editor.scroll_top()` | `state.tab_manager.active_editor().map(\|e\| e.scroll_top()).unwrap_or(0)` |
| `state.editor.total_lines()` | `state.tab_manager.active_editor().map(\|e\| e.total_lines()).unwrap_or(1)` |
| `state.editor.cursor_line()` | `state.tab_manager.active_editor().map(\|e\| e.cursor_line()).unwrap_or(0)` |
| `state.editor.cursor_col()` | `state.tab_manager.active_editor().map(\|e\| e.cursor_col()).unwrap_or(0)` |
| `state.editor.buffer.rope()` | `state.tab_manager.active_editor().map(\|e\| e.buffer.rope())` — needs special handling |
| `state.editor.insert_char(ch)` | `if let Some(ed) = state.tab_manager.active_editor_mut() { ed.insert_char(ch); }` |
| `state.editor.backspace()` | `if let Some(ed) = state.tab_manager.active_editor_mut() { ed.backspace(); }` |
| `state.editor.move_left()` | `if let Some(ed) = state.tab_manager.active_editor_mut() { ed.move_left(); }` |

For the render loop where you need both read AND buffer access, use a pattern like:
```rust
if let Some(editor) = state.tab_manager.active_editor() {
    let scroll_top = editor.scroll_top();
    let total_lines = editor.total_lines();
    // ... all the rendering code that uses editor ...
} else {
    // No file open — render welcome text
}
```

**ALSO update rehighlight() calls:**
```rust
// OLD: state.editor.rehighlight();
// NEW: if let Some(ed) = state.tab_manager.active_editor_mut() { ed.rehighlight(); }
```

## TASK 2: Wire Tab Switching Keyboard Shortcuts

**Add to keyboard handler:**

```rust
// Ctrl+Tab → next tab
Key::Named(NamedKey::Tab) if ctrl => {
    state.tab_manager.next_tab();
    // Update tab bar visual
    state.window.request_redraw();
}
// Ctrl+Shift+Tab → previous tab
// Ctrl+W → close current tab
Key::Character(ref c) if c == "w" && ctrl => {
    state.tab_manager.close_current();
    state.window.request_redraw();
}
```

**Update tab bar rendering** to reflect `tab_manager.tabs` instead of the old `tab_bar.tabs`:
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

## TASK 3: Build Real File Explorer

**Create `crates/forge-app/src/file_explorer.rs`:**

```rust
use crate::file_tree_ui::DisplayNode;
use std::path::{Path, PathBuf};

pub struct FileExplorer {
    pub root: Option<PathBuf>,
    pub nodes: Vec<DisplayNode>,
    pub paths: Vec<PathBuf>,  // parallel to nodes — actual paths
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
            // Skip hidden and common excluded dirs
            if name.starts_with('.') || name == "target" || name == "node_modules" { continue; }
            let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
            let path = entry.path();
            self.nodes.push(DisplayNode { label: name, depth, is_dir, expanded: depth == 0 && is_dir });
            self.paths.push(path.clone());
            // Auto-expand first level
            if is_dir && depth == 0 {
                let _ = self.scan_recursive(&path, depth + 1);
            }
        }
        Ok(())
    }

    /// Get the path of the clicked node
    pub fn get_path(&self, index: usize) -> Option<&Path> {
        self.paths.get(index).map(|p| p.as_path())
    }

    /// Toggle directory expand/collapse
    pub fn toggle_expand(&mut self, index: usize) {
        if let Some(node) = self.nodes.get_mut(index) {
            if node.is_dir {
                node.expanded = !node.expanded;
                // TODO: rescan children if expanding
            }
        }
    }
}
```

Tests (≥2): scan finds files, paths parallel to nodes.

**Add `mod file_explorer;` to `main.rs`.**

## TASK 4: Wire File Explorer to Sidebar Rendering

**In `application.rs`:**

Add `file_explorer: crate::file_explorer::FileExplorer` and `file_tree_ui: crate::file_tree_ui::FileTreeUi` to `AppState`.

Initialize in `init_state()`:
```rust
let mut file_explorer = crate::file_explorer::FileExplorer::new();
let cwd = std::env::current_dir().unwrap_or_default();
let _ = file_explorer.scan_directory(&cwd);
let file_tree_ui = crate::file_tree_ui::FileTreeUi::new();
```

**Replace hardcoded sidebar text** with dynamic file tree rendering:
```rust
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

**Wire sidebar clicks:**
```rust
// In handle_input(), when click is in sidebar zone:
if let Some(ref sb) = state.layout.sidebar {
    if sb.contains(mx, my) {
        let line_h = 22.0;
        let click_index = ((my - sb.y - 40.0) / line_h) as usize; // offset for header
        if click_index < state.file_explorer.nodes.len() {
            let node = &state.file_explorer.nodes[click_index];
            if node.is_dir {
                state.file_explorer.toggle_expand(click_index);
            } else if let Some(path) = state.file_explorer.get_path(click_index) {
                let path_str = path.to_string_lossy().to_string();
                let _ = state.tab_manager.open_file(&path_str);
            }
        }
    }
}
```

## TASK 5: Render Minimap

**Add minimap zone to `LayoutZones`** in `ui.rs` — a narrow strip (60px wide) on the right edge of the editor.

**Add minimap rendering to render loop:**
```rust
if let Some(ref minimap_zone) = state.layout.minimap {
    if let Some(editor) = state.tab_manager.active_editor() {
        let visible_lines = (state.layout.editor.height / LayoutConstants::LINE_HEIGHT) as usize;
        let minimap = crate::minimap::Minimap::build(
            editor.total_lines(),
            editor.scroll_top(),
            editor.scroll_top() + visible_lines,
        );
        // Render minimap background
        state.render_batch.push(crate::rect_renderer::RectPrimitive {
            x: minimap_zone.x, y: minimap_zone.y,
            width: minimap_zone.width, height: minimap_zone.height,
            color: [0.09, 0.10, 0.14, 1.0],
            border_radius: 0.0,
        });
        // Render viewport indicator
        let total = editor.total_lines().max(1) as f32;
        let vp_y = minimap_zone.y + (editor.scroll_top() as f32 / total) * minimap_zone.height;
        let vp_h = (visible_lines as f32 / total) * minimap_zone.height;
        state.render_batch.push(crate::rect_renderer::RectPrimitive {
            x: minimap_zone.x, y: vp_y,
            width: minimap_zone.width, height: vp_h.max(10.0),
            color: [1.0, 1.0, 1.0, 0.1],
            border_radius: 0.0,
        });
    }
}
```

## TASK 6: Render Code Folding Markers in Gutter

**Add `folding_manager: crate::code_folding::FoldingManager` to `AppState`.**

Initialize: `let folding_manager = crate::code_folding::FoldingManager::new();`

**In the gutter rendering loop**, add fold markers:
```rust
// After drawing line numbers, check for fold ranges on each line
if let Some(editor) = state.tab_manager.active_editor() {
    state.folding_manager.compute_ranges(&editor.buffer.text());
    for range in &state.folding_manager.ranges {
        if range.start_line >= scroll_top && range.start_line < scroll_top + vis_lines {
            let y = state.layout.gutter.y + ((range.start_line - scroll_top) as f32 * LayoutConstants::LINE_HEIGHT);
            let marker = if range.collapsed { "▶" } else { "▼" };
            // Append fold marker to gutter text for this line
            // (This modifies the gutter text string before set_text)
        }
    }
}
```

## TASK 7: Render Indent Guides as Vertical Lines

**In the render loop**, after the editor text rendering, add indent guide lines:
```rust
if let Some(editor) = state.tab_manager.active_editor() {
    let guides = crate::indent_guides::IndentGuides::compute(
        &editor.buffer.text(),
        4, // tab_size
        editor.cursor_line(),
    );
    let char_width = 8.5; // approximate monospace character width
    for guide in &guides {
        if guide.line >= scroll_top && guide.line < scroll_top + vis_lines {
            let x = state.layout.editor.x + (guide.column as f32 * char_width);
            let y = state.layout.editor.y + ((guide.line - scroll_top) as f32 * LayoutConstants::LINE_HEIGHT);
            let color = if guide.is_active {
                [1.0, 1.0, 1.0, 0.2]
            } else {
                [1.0, 1.0, 1.0, 0.06]
            };
            state.render_batch.push(crate::rect_renderer::RectPrimitive {
                x, y,
                width: 1.0,
                height: LayoutConstants::LINE_HEIGHT,
                color,
                border_radius: 0.0,
            });
        }
    }
}
```

## TASK 8: Update Window Title and Status Bar

**Update window title** to show active file:
```rust
let title = if let Some(editor) = state.tab_manager.active_editor() {
    editor.window_title()
} else {
    "Forge — [no file]".to_string()
};
state.window.set_title(&format!("{} - {}", title, self.current_mode.label()));
```

**Update status bar** to show file language, encoding, branch:
```rust
let language = state.tab_manager.tabs.get(state.tab_manager.active)
    .and_then(|t| t.path.as_ref())
    .map(|p| {
        let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
        match ext {
            "rs" => "Rust",
            "js" | "jsx" => "JavaScript",
            "ts" | "tsx" => "TypeScript",
            "py" => "Python",
            "json" => "JSON",
            "toml" => "TOML",
            "md" => "Markdown",
            "html" => "HTML",
            "css" => "CSS",
            _ => "Plain Text",
        }
    })
    .unwrap_or("Plain Text");

let status_text = format!(
    "  Forge IDE  │  Ln {}, Col {}  │  UTF-8  │  {}  │  {} fps  │  {}  │  {} tabs",
    cursor_line, cursor_col, language, fps, mode.label(), state.tab_manager.tab_count()
);
```

---

## FINAL VERIFICATION

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test --workspace
cargo check --workspace
```

ALL FOUR must exit 0. If `state.editor` references remain and cause compile errors, search for ALL of them and replace with TabManager access.
