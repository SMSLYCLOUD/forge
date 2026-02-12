# SESSION 6 — Wire ALL UI Components into Render Loop + Full Input Handling
# ONE JULES TASK — Copy this ENTIRE file as one Jules prompt.
# PREREQUISITE: Session 3 Redo must be merged first. `cargo check --workspace` exits 0.
# THIS SESSION MODIFIES `application.rs` — THE ONLY SESSION ALLOWED TO DO SO.
# ═══════════════════════════════════════════════════════════════

You are working on **Forge**, a GPU-accelerated code editor written in Rust. The editor has ~60 modules in `crates/forge-app/src/` that define UI components (FindBar, CommandPalette, GitPanel, SearchPanel, Autocomplete, etc.) but they are NOT yet connected to the render loop or input handling.

**Your job: wire every existing UI component into real rendering and input handling in `application.rs`.**

## CRITICAL CONTEXT — READ ALL OF THIS

### How the render loop works (lines ~380-900 of `application.rs`)

The `render_frame()` function follows this exact pattern:
1. **Collect rectangles** — each UI component has a `render_rects(&Zone) -> Vec<RectPrimitive>` method
2. **Upload rectangles** — `state.rect_renderer.prepare(&gpu.queue, &rects)`
3. **Set text content** — each UI zone has a glyphon `Buffer` using `set_text()` or `set_rich_text()`
4. **Create TextAreas** — each zone mapped to a `TextArea` with bounds
5. **Submit to GPU** — `text_renderer.prepare()` + rect_renderer + pass.draw()

### Current struct layout

```rust
// Application struct owns config/theme and has state: Option<AppState>
pub struct Application {
    file_path: Option<String>,
    state: Option<AppState>,
    modifiers: ModifiersState,
    current_mode: UiMode,
    config: forge_config::ForgeConfig,
    theme: forge_theme::Theme,
    // COMMENTED OUT — YOU MUST UNCOMMENT AND WIRE:
    // find_bar: crate::find_bar::FindBar,
    // command_palette: crate::command_palette::CommandPalette,
    // ... etc
}

// AppState struct owns GPU, text rendering, editor, and UI components
struct AppState {
    window: Arc<Window>,
    gpu: GpuContext,
    font_system: FontSystem,
    swash_cache: SwashCache,
    cache: Cache,
    text_atlas: TextAtlas,
    viewport: Viewport,
    text_renderer: TextRenderer,
    editor_buffer: GlyphonBuffer,
    gutter_buffer: GlyphonBuffer,
    tab_buffer: GlyphonBuffer,
    status_buffer: GlyphonBuffer,
    breadcrumb_buffer: GlyphonBuffer,
    sidebar_buffer: GlyphonBuffer,
    editor: Editor,
    rect_renderer: RectRenderer,
    layout: LayoutZones,
    tab_bar: TabBar,
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

### Key types/imports already available:
```
use glyphon::{Attrs, Buffer as GlyphonBuffer, Color as GlyphonColor, Family, FontSystem, Metrics, Shaping, TextArea, TextBounds};
```

### LayoutZones (from `ui.rs`):
The `LayoutZones` struct has these zones as `Zone` structs (x, y, width, height + `contains(mx, my)` method):
- `editor`, `gutter`, `tab_bar`, `status_bar`, `activity_bar`, `breadcrumb_bar`, `scrollbar_v`
- `sidebar: Option<Zone>`
- `bottom_panel: Option<Zone>` — NOT YET PRESENT, YOU MUST ADD

### Zone struct:
```rust
pub struct Zone { pub x: f32, pub y: f32, pub width: f32, pub height: f32 }
impl Zone { pub fn contains(&self, mx: f32, my: f32) -> bool { ... } }
```

---

## RULES (MANDATORY)

1. No `.unwrap()` in production. Use `if let`, `match`, or `.unwrap_or_default()`.
2. `cargo fmt` + `cargo clippy -- -D warnings` = zero warnings.
3. Do NOT delete existing rendering code. ADD to it.
4. Every feature panel must have a keyboard shortcut to toggle visibility.
5. Do NOT add new crate dependencies — all needed deps are already in Cargo.toml.
6. When a module struct doesn't exist or its API doesn't match, adapt gracefully (use `Default::default()`, skip rendering, etc.)

---

## TASK 1: Add Bottom Panel Zone to LayoutZones

**Modify `crates/forge-app/src/ui.rs`:**

Add `bottom_panel: Option<Zone>` field to `LayoutZones`.

In the `compute()` function, add logic: if a bottom panel should be shown (pass a new bool param `bottom_panel_open: bool`), allocate 200px at the bottom of the editor area for the bottom panel zone. Reduce editor height by 200px when bottom panel is open.

Update ALL calls to `LayoutZones::compute()` throughout `application.rs` to include the new parameter (default `false`).

## TASK 2: Add Overlay Text Buffers to AppState

**Modify `application.rs` — add new fields to `AppState`:**

```rust
// Overlay text buffers
find_buffer: GlyphonBuffer,
command_palette_buffer: GlyphonBuffer,
bottom_panel_buffer: GlyphonBuffer,
```

Initialize them in `init_state()` the same way other buffers are created:
```rust
let find_buffer = GlyphonBuffer::new(&mut font_system, Metrics::new(LayoutConstants::SMALL_FONT_SIZE, LayoutConstants::LINE_HEIGHT));
// same for command_palette_buffer and bottom_panel_buffer
```

## TASK 3: Uncomment and Initialize UI Panel Fields

**Modify `application.rs` — uncomment fields in `Application` struct:**

Change the commented-out fields to real fields:
```rust
pub struct Application {
    file_path: Option<String>,
    state: Option<AppState>,
    modifiers: ModifiersState,
    current_mode: UiMode,
    config: forge_config::ForgeConfig,
    theme: forge_theme::Theme,
    find_bar: crate::find_bar::FindBar,
    command_palette: crate::command_palette::CommandPalette,
    bottom_panel: crate::bottom_panel::BottomPanel,
    notifications: crate::notifications::NotificationManager,
    file_picker: crate::file_picker::FilePicker,
    context_menu: crate::context_menu::ContextMenu,
    zen_mode: crate::zen_mode::ZenMode,
    git_panel: crate::git_panel::GitPanel,
    search_panel: crate::search_panel::SearchPanel,
    autocomplete: crate::autocomplete::AutocompleteMenu,
    outline_panel: crate::outline_panel::OutlinePanel,
}
```

In `Application::new()`, initialize all fields. Use `.default()` if the struct implements Default, otherwise use `::new()`. Check each module's actual constructor:
- `FindBar` → likely `FindBar::default()` or `FindBar::new()`
- `CommandPalette` → likely `CommandPalette::new()`
- `BottomPanel` → `BottomPanel::default()` or `::new()`
- etc.

**IMPORTANT:** Read each module file FIRST to find the actual constructor. If a module has `#[derive(Default)]` or `impl Default`, use `.default()`. If it has `fn new() -> Self`, use `::new()`. If neither exists, add `#[derive(Default)]` to the struct in that module file.

## TASK 4: Wire Keyboard Shortcuts for Panel Toggles

**Modify `application.rs` keyboard handler** (the `Key::Character` match arms in `window_event()`):

Add these shortcut handlers IN the existing `if ctrl { match key { ... } }` block:

```rust
// Ctrl+F → Toggle Find Bar
Key::Character(ref c) if c == "f" => {
    self.find_bar.toggle();
    state.window.request_redraw();
}
// Ctrl+H → Toggle Replace (find bar in replace mode)
Key::Character(ref c) if c == "h" => {
    // If find_bar has a replace mode, toggle it
    self.find_bar.toggle();
    state.window.request_redraw();
}
// Ctrl+Shift+P → Toggle Command Palette
// (Need to check shift modifier)
Key::Character(ref c) if c == "P" && self.modifiers.shift_key() => {
    self.command_palette.toggle();
    state.window.request_redraw();
}
// Ctrl+P → Toggle File Picker
Key::Character(ref c) if c == "p" && !self.modifiers.shift_key() => {
    self.file_picker.toggle();
    state.window.request_redraw();
}
// Ctrl+G → Toggle Go To Line
Key::Character(ref c) if c == "g" => {
    // Toggle go_to_line if it exists as a module
    state.window.request_redraw();
}
// Ctrl+Shift+F → Toggle Search Panel
Key::Character(ref c) if c == "F" && self.modifiers.shift_key() => {
    self.search_panel.toggle();
    state.window.request_redraw();
}
// Ctrl+` → Toggle Bottom Panel (Terminal)
Key::Character(ref c) if c == "`" => {
    self.bottom_panel.toggle();
    let (w, h) = state.gpu.size();
    state.layout = LayoutZones::compute(w as f32, h as f32, state.sidebar_open, state.ai_panel_open, self.bottom_panel.visible);
    state.window.request_redraw();
}
// Ctrl+B → Toggle Sidebar
Key::Character(ref c) if c == "b" => {
    state.sidebar_open = !state.sidebar_open;
    let (w, h) = state.gpu.size();
    state.layout = LayoutZones::compute(w as f32, h as f32, state.sidebar_open, state.ai_panel_open, self.bottom_panel.visible);
    state.window.request_redraw();
}
```

Also add Escape handler to close overlays:
```rust
Key::Named(NamedKey::Escape) => {
    if self.command_palette.visible { self.command_palette.close(); }
    else if self.find_bar.visible { self.find_bar.close(); }
    else if self.file_picker.visible { self.file_picker.close(); }
    else if self.context_menu.visible { self.context_menu.close(); }
    else if self.autocomplete.visible { self.autocomplete.hide(); }
    state.window.request_redraw();
}
```

**IMPORTANT:** Check each module struct for the actual field name for visibility (likely `visible: bool`) and method names (`toggle()`, `open()`, `close()`, `is_visible()`, `show()`, `hide()`). Read the module file FIRST, then use the correct method name.

## TASK 5: Render Find Bar Overlay

**Add to the render loop in `render_frame()`** after the editor text content section:

```rust
// ── Find Bar Overlay ──
if self.find_bar.visible {
    // Draw find bar background rectangle at top of editor zone
    let find_zone = Zone {
        x: state.layout.editor.x,
        y: state.layout.editor.y,
        width: state.layout.editor.width.min(500.0),
        height: 35.0,
    };
    state.render_batch.push(crate::rect_renderer::RectPrimitive {
        x: find_zone.x,
        y: find_zone.y,
        width: find_zone.width,
        height: find_zone.height,
        color: [0.18, 0.20, 0.25, 1.0],
        border_radius: 0.0,
    });

    // Draw find bar text (the search query)
    let find_query = &self.find_bar.query; // or however the query field is named
    let find_text = format!("🔍 {}", if find_query.is_empty() { "Find..." } else { find_query });
    state.find_buffer.set_size(&mut state.font_system, Some(find_zone.width - 20.0), Some(find_zone.height));
    state.find_buffer.set_text(
        &mut state.font_system,
        &find_text,
        Attrs::new().family(Family::SansSerif).color(GlyphonColor::rgb(220, 220, 220)),
        Shaping::Advanced,
    );
    state.find_buffer.shape_until_scroll(&mut state.font_system, false);
    // Add TextArea for find buffer to the text_areas vec
}
```

**BEFORE writing ANY of this code:** Read `crates/forge-app/src/find_bar.rs` to check the actual struct fields and method names. Use whatever the real field name is for the search query string.

## TASK 6: Render Command Palette Overlay

**Add to render loop:**

Command palette renders as a centered overlay box in the upper third of the screen.

```rust
if self.command_palette.visible {
    let palette_width = 600.0_f32.min(state.layout.editor.width * 0.8);
    let palette_x = state.layout.editor.x + (state.layout.editor.width - palette_width) / 2.0;
    let palette_y = state.layout.editor.y + 50.0;
    let palette_height = 300.0;

    // Background rect
    state.render_batch.push(crate::rect_renderer::RectPrimitive {
        x: palette_x,
        y: palette_y,
        width: palette_width,
        height: palette_height,
        color: [0.15, 0.17, 0.22, 0.98],
        border_radius: 0.0,
    });

    // Text content — show input + filtered commands
    let palette_text = self.command_palette.render_text(); // or build text from items
    // ... set_text on command_palette_buffer ...
}
```

**Read `command_palette.rs` FIRST** to check actual API.

## TASK 7: Render Bottom Panel (Terminal / Output / Problems)

**Add to render loop:**

If bottom panel is open and its zone exists, render terminal grid or output text:

```rust
if let Some(ref bp_zone) = state.layout.bottom_panel {
    if self.bottom_panel.visible {
        // Background
        state.render_batch.push(crate::rect_renderer::RectPrimitive {
            x: bp_zone.x, y: bp_zone.y,
            width: bp_zone.width, height: bp_zone.height,
            color: [0.10, 0.11, 0.15, 1.0],
            border_radius: 0.0,
        });

        // Tab header for bottom panel (Terminal | Output | Problems)
        let tab_text = "  Terminal  │  Output  │  Problems";
        state.bottom_panel_buffer.set_size(&mut state.font_system, Some(bp_zone.width), Some(bp_zone.height));
        state.bottom_panel_buffer.set_text(
            &mut state.font_system,
            tab_text,
            Attrs::new().family(Family::Monospace).color(GlyphonColor::rgb(180, 180, 180)),
            Shaping::Advanced,
        );
        state.bottom_panel_buffer.shape_until_scroll(&mut state.font_system, false);
    }
}
```

## TASK 8: Render Git Panel in Sidebar

**Modify the sidebar rendering** (currently hardcoded file tree at line ~693):

Replace the hardcoded sidebar text with dynamic content:

```rust
let sidebar_text = if state.sidebar_open {
    // Check which sidebar view is active
    if self.git_panel.visible {
        // Render git status
        let mut text = "  SOURCE CONTROL\n\n".to_string();
        for file in &self.git_panel.files {
            text.push_str(&format!("  {} {}\n", file.status, file.path));
        }
        if self.git_panel.files.is_empty() {
            text.push_str("  No changes\n");
        }
        text
    } else if self.search_panel.visible {
        // Render search results
        let mut text = format!("  SEARCH: {}\n\n", self.search_panel.query);
        for result in &self.search_panel.results {
            text.push_str(&format!("  {}:{} {}\n", result.file, result.line, result.text.trim()));
        }
        if self.search_panel.results.is_empty() && !self.search_panel.query.is_empty() {
            text.push_str("  No results found\n");
        }
        text
    } else if self.outline_panel.visible {
        // Render outline
        let mut text = "  OUTLINE\n\n".to_string();
        for sym in &self.outline_panel.symbols {
            text.push_str(&format!("  {:?} {} (line {})\n", sym.kind, sym.name, sym.line));
        }
        text
    } else {
        // Default: file explorer
        "  EXPLORER\n\n  📁 src\n    📄 main.rs\n    📄 editor.rs\n    📄 gpu.rs\n    📄 ui.rs\n  📁 crates\n  📄 Cargo.toml\n  📄 README.md".to_string()
    }
} else {
    String::new()
};
```

**Read** `git_panel.rs`, `search_panel.rs`, and `outline_panel.rs` FIRST to check actual field names.

## TASK 9: Wire Activity Bar Items to Sidebar Panels

**Modify `handle_input()` in `application.rs`:**

The activity bar click handler currently only handles `AiAgent`. Extend it to toggle sidebar panels:

```rust
if let Some(item) = state.activity_bar.handle_click(my, &state.layout.activity_bar) {
    match item {
        crate::activity_bar::ActivityItem::Explorer => {
            // Close all sidebar panels, show file explorer
            self.git_panel.visible = false;
            self.search_panel.visible = false;
            self.outline_panel.visible = false;
            state.sidebar_open = !state.sidebar_open;
        }
        crate::activity_bar::ActivityItem::Search => {
            self.search_panel.toggle();
            state.sidebar_open = self.search_panel.visible;
            self.git_panel.visible = false;
            self.outline_panel.visible = false;
        }
        crate::activity_bar::ActivityItem::Git => {
            self.git_panel.toggle();
            state.sidebar_open = self.git_panel.visible;
            self.search_panel.visible = false;
            self.outline_panel.visible = false;
        }
        crate::activity_bar::ActivityItem::AiAgent => {
            state.ai_panel_open = !state.ai_panel_open;
        }
        _ => {}
    }
    let (w, h) = state.gpu.size();
    state.layout = LayoutZones::compute(w as f32, h as f32, state.sidebar_open, state.ai_panel_open, self.bottom_panel.visible);
}
```

**Read `crates/forge-app/src/activity_bar.rs` FIRST** to check the actual `ActivityItem` enum variants.

## TASK 10: Final TextArea Assembly + Autocomplete Render

**Modify the TextArea assembly** (the section that creates the `text_areas` Vec):

Add TextArea entries for the new overlays:

```rust
let mut text_areas = vec![/* existing areas */];

// Add find bar TextArea if visible
if self.find_bar.visible {
    text_areas.push(TextArea {
        buffer: &state.find_buffer,
        left: state.layout.editor.x + 10.0,
        top: state.layout.editor.y + 6.0,
        scale: 1.0,
        bounds: TextBounds {
            left: state.layout.editor.x as i32 + 10,
            top: state.layout.editor.y as i32 + 6,
            right: (state.layout.editor.x + 500.0) as i32,
            bottom: (state.layout.editor.y + 35.0) as i32,
        },
        default_color: GlyphonColor::rgb(220, 220, 220),
        custom_glyphs: &[],
    });
}

// Add command palette TextArea if visible
if self.command_palette.visible {
    let pw = 600.0_f32.min(state.layout.editor.width * 0.8);
    let px = state.layout.editor.x + (state.layout.editor.width - pw) / 2.0;
    let py = state.layout.editor.y + 50.0;
    text_areas.push(TextArea {
        buffer: &state.command_palette_buffer,
        left: px + 10.0,
        top: py + 6.0,
        scale: 1.0,
        bounds: TextBounds {
            left: px as i32 + 10,
            top: py as i32 + 6,
            right: (px + pw) as i32,
            bottom: (py + 300.0) as i32,
        },
        default_color: GlyphonColor::rgb(200, 200, 200),
        custom_glyphs: &[],
    });
}

// Bottom panel TextArea
if let Some(ref bp) = state.layout.bottom_panel {
    if self.bottom_panel.visible {
        text_areas.push(TextArea {
            buffer: &state.bottom_panel_buffer,
            left: bp.x + 10.0,
            top: bp.y + 6.0,
            scale: 1.0,
            bounds: TextBounds {
                left: bp.x as i32,
                top: bp.y as i32,
                right: (bp.x + bp.width) as i32,
                bottom: (bp.y + bp.height) as i32,
            },
            default_color: GlyphonColor::rgb(180, 180, 180),
            custom_glyphs: &[],
        });
    }
}
```

**Also add autocomplete rendering** — when typing, show autocomplete popup near cursor:

```rust
if self.autocomplete.visible {
    // Draw completion popup background
    let cursor_x = state.layout.editor.x + (state.editor.cursor_col() as f32 * 8.5); // approximate char width
    let cursor_y = state.layout.editor.y + ((state.editor.cursor_line() - state.editor.scroll_top()) as f32 * LayoutConstants::LINE_HEIGHT) + LayoutConstants::LINE_HEIGHT;
    let popup_width = 300.0;
    let popup_height = (self.autocomplete.items.len().min(8) as f32) * LayoutConstants::LINE_HEIGHT + 10.0;

    state.render_batch.push(crate::rect_renderer::RectPrimitive {
        x: cursor_x, y: cursor_y,
        width: popup_width, height: popup_height,
        color: [0.15, 0.17, 0.22, 0.98],
        border_radius: 0.0,
    });
    // Items text would go to a new buffer — or reuse an existing one
}
```

**Read autocomplete.rs FIRST** to check `items` field and `visible` field names.

---

## SELF-CHECK BEFORE FINISHING

For every field you reference on `self.*` or `state.*`:
1. Verify the struct HAS that field by reading the source file
2. Verify the field type matches what you're doing with it
3. Verify the method you call (`toggle()`, `visible`, `close()`, etc.) EXISTS on that struct

For every new parameter added to a function:
1. Find ALL call sites and update them

For every glyphon Buffer:
1. Verify it's created in `init_state()`
2. Verify it's included in the `AppState` struct
3. Verify `set_text()` is called before it's used in a `TextArea`

## FINAL VERIFICATION

```bash
cargo fmt --check        # Must exit 0
cargo clippy -- -D warnings  # Must exit 0
cargo test --workspace   # Must exit 0
cargo check --workspace  # Must exit 0
```

**ALL FOUR must exit 0.** If any module's API doesn't match what you expected, adapt your code to match reality. Read the files FIRST, code SECOND.
