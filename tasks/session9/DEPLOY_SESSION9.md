# SESSION 9 — Theme UI + Settings + Extensions + Notifications + Final Polish
# ONE JULES TASK — Copy this ENTIRE file as one Jules prompt.
# PREREQUISITE: Session 8 must be merged. `cargo check --workspace` exits 0.
# THIS SESSION MODIFIES `application.rs` AND SEVERAL MODULES.
# ═══════════════════════════════════════════════════════════════

You are working on **Forge**, a GPU-accelerated code editor written in Rust. This is the FINAL session. After this, the editor should be feature-complete with VS Code parity.

## CRITICAL CONTEXT

### Existing modules you MUST wire (read each file FIRST):

| Module | Key API |
|--------|---------|
| `settings_ui.rs` | `SettingsUi` struct with `visible`, `toggle()`, sections, render data |
| `extensions_panel.rs` | `ExtensionsPanel` struct with `visible`, `toggle()`, extension list |
| `notifications.rs` | `NotificationManager` struct with `push()`, `dismiss()`, `active()` |
| `context_menu.rs` | `ContextMenu` struct with `visible`, `show()`, `hide()`, `items` |
| `hover_info.rs` | `HoverInfo` struct with `visible`, `text`, `position` |
| `debug_ui.rs` | `DebugUi` — debug overlay |
| `drag_drop.rs` | `DragDrop` — drag state tracking |
| `comment_toggle.rs` | `toggle_comment()` function |
| `word_wrap.rs` | `WordWrap` — soft wrap logic |
| `selection_render.rs` | Selection highlight rendering helpers |

### `forge-theme` crate:
```rust
pub struct Theme { pub name: String, pub colors: ThemeColors }
pub struct ThemeColors {
    pub background: [u8; 3],
    pub foreground: [u8; 3],
    pub selection: [u8; 3],
    pub cursor: [u8; 3],
    pub line_highlight: [u8; 3],
    pub sidebar_bg: [u8; 3],
    pub statusbar_bg: [u8; 3],
    // ... more color fields
}
```

### `forge-config` crate:
```rust
pub struct ForgeConfig {
    pub font_size: f32,
    pub tab_size: u32,
    pub word_wrap: bool,
    pub auto_save: bool,
    pub minimap_enabled: bool,
    pub theme_name: String,
    // ... more config fields
}
```

---

## RULES (MANDATORY)

1. No `.unwrap()` in production.
2. `cargo fmt` + `cargo clippy -- -D warnings` = zero warnings.
3. **Read each module file FIRST** before referencing its API.
4. All colors in the render loop must come from `self.theme.colors.*` — NO hardcoded RGB values. Replace ALL existing hardcoded colors.

---

## TASK 1: Theme-Aware Rendering — Remove All Hardcoded Colors

**This is the most impactful task.** Go through ALL rendering code in `application.rs` and replace every hardcoded color with theme values.

First, **read `crates/forge-theme/src/lib.rs`** to find the actual `ThemeColors` fields.

Then replace ALL hardcoded colors:

```rust
// BEFORE (scattered throughout render loop):
color: [0.12, 0.12, 0.18, 1.0]  // editor background
color: GlyphonColor::rgb(248, 248, 242)  // text
color: [0.10, 0.11, 0.15, 1.0]  // sidebar background

// AFTER (using theme):
let bg = self.theme.colors.background;
color: [bg[0] as f32 / 255.0, bg[1] as f32 / 255.0, bg[2] as f32 / 255.0, 1.0]
// For GlyphonColor:
let fg = self.theme.colors.foreground;
GlyphonColor::rgb(fg[0], fg[1], fg[2])
```

Create a helper method for the conversion:
```rust
fn theme_color(rgb: [u8; 3]) -> [f32; 4] {
    [rgb[0] as f32 / 255.0, rgb[1] as f32 / 255.0, rgb[2] as f32 / 255.0, 1.0]
}
fn theme_glyph_color(rgb: [u8; 3]) -> GlyphonColor {
    GlyphonColor::rgb(rgb[0], rgb[1], rgb[2])
}
```

Replace at minimum these locations:
- Editor background → `theme.colors.background`
- Editor text → `theme.colors.foreground`
- Sidebar background → `theme.colors.sidebar_bg`
- Status bar background → `theme.colors.statusbar_bg`
- Tab bar background and text
- Gutter text color
- Current line highlight → `theme.colors.line_highlight`
- Cursor color → `theme.colors.cursor`
- Selection color → `theme.colors.selection`
- Breadcrumb text
- Find bar background
- Command palette background
- Bottom panel background

## TASK 2: Settings UI Rendering

**Read `settings_ui.rs` FIRST.**

Add `settings_ui: crate::settings_ui::SettingsUi` to `Application` struct. Initialize in `new()`.

Wire `Ctrl+,` shortcut:
```rust
Key::Character(ref c) if c == "," && ctrl => {
    self.settings_ui.toggle();
    state.window.request_redraw();
}
```

Render settings as full-screen overlay when visible:
```rust
if self.settings_ui.visible {
    let zone = &state.layout.editor;
    // Semi-transparent overlay background
    state.render_batch.push(RectPrimitive {
        x: zone.x, y: zone.y, width: zone.width, height: zone.height,
        color: Application::theme_color(self.theme.colors.background),
        border_radius: 0.0,
    });

    // Build settings text from config
    let settings_text = format!(
        "  ⚙ Settings\n\n\
         Font Size: {}\n\
         Tab Size: {}\n\
         Word Wrap: {}\n\
         Auto Save: {}\n\
         Minimap: {}\n\
         Theme: {}\n",
        self.config.font_size,
        self.config.tab_size,
        if self.config.word_wrap { "On" } else { "Off" },
        if self.config.auto_save { "On" } else { "Off" },
        if self.config.minimap_enabled { "On" } else { "Off" },
        self.config.theme_name,
    );

    // Render settings text in editor zone
    // Reuse an existing buffer or create a settings_buffer
}
```

## TASK 3: Notifications Rendering

**Read `notifications.rs` FIRST.**

Add `notifications: crate::notifications::NotificationManager` to `Application` struct.

Render notifications as toast popups in the bottom-right corner:
```rust
// In render loop, after all other rendering:
let active_notifications = self.notifications.active();
for (i, notif) in active_notifications.iter().enumerate().take(3) {
    let notif_width = 350.0;
    let notif_height = 60.0;
    let x = state.layout.editor.x + state.layout.editor.width - notif_width - 20.0;
    let y = state.layout.status_bar.y - ((i + 1) as f32 * (notif_height + 8.0));

    // Notification background
    let bg_color = match notif.severity {
        // Read the actual severity enum/field from notifications.rs
        _ => [0.15, 0.17, 0.22, 0.95],
    };
    state.render_batch.push(RectPrimitive {
        x, y, width: notif_width, height: notif_height,
        color: bg_color,
        border_radius: 0.0,
    });
    // Notification text would need a buffer — reuse or create
}
```

## TASK 4: Context Menu Rendering

**Read `context_menu.rs` FIRST.**

Add `context_menu: crate::context_menu::ContextMenu` to `Application`.

Wire right-click to show context menu:
```rust
// In handle_input(), for right mouse button:
winit::event::MouseButton::Right => {
    if let Some((mx, my)) = state.last_mouse_position {
        if state.layout.editor.contains(mx, my) {
            self.context_menu.show(mx, my, vec![
                "Cut".into(),
                "Copy".into(),
                "Paste".into(),
                "---".into(), // separator
                "Go to Definition".into(),
                "Find References".into(),
                "Rename Symbol".into(),
                "---".into(),
                "Command Palette...".into(),
            ]);
        }
    }
}
```

Render context menu:
```rust
if self.context_menu.visible {
    let cx = self.context_menu.x; // or however position is stored
    let cy = self.context_menu.y;
    let menu_width = 220.0;
    let item_height = 28.0;
    let menu_height = self.context_menu.items.len() as f32 * item_height;

    // Background
    state.render_batch.push(RectPrimitive {
        x: cx, y: cy, width: menu_width, height: menu_height,
        color: [0.18, 0.20, 0.25, 0.98],
        border_radius: 0.0,
    });

    // Menu items text
    let menu_text = self.context_menu.items.iter()
        .map(|item| format!("  {}", item))
        .collect::<Vec<_>>().join("\n");
    // Set text on a buffer, create TextArea
}
```

**Read context_menu.rs FIRST** to get actual field names for position (x/y) and items.

## TASK 5: Selection Highlight Rendering

**Read `selection_render.rs` FIRST.**

Wire visual selection rendering:
```rust
if let Some(editor) = state.tab_manager.active_editor() {
    if editor.has_selection() {
        // Get selection range from editor
        let (sel_start, sel_end) = editor.selection_range();
        // Highlight selected lines/cols
        for line in sel_start.line..=sel_end.line {
            if line >= scroll_top && line < scroll_top + vis_lines {
                let start_col = if line == sel_start.line { sel_start.col } else { 0 };
                let end_col = if line == sel_end.line { sel_end.col } else { /* line length */ 999 };
                let x = state.layout.editor.x + (start_col as f32 * 8.5);
                let y = state.layout.editor.y + ((line - scroll_top) as f32 * LayoutConstants::LINE_HEIGHT);
                let width = ((end_col - start_col) as f32 * 8.5).max(8.5);
                let sel_color = self.theme.colors.selection;
                state.render_batch.push(RectPrimitive {
                    x, y,
                    width, height: LayoutConstants::LINE_HEIGHT,
                    color: [sel_color[0] as f32 / 255.0, sel_color[1] as f32 / 255.0, sel_color[2] as f32 / 255.0, 0.3],
                    border_radius: 0.0,
                });
            }
        }
    }
}
```

**Read editor.rs FIRST** to check if `has_selection()` and `selection_range()` exist. If they don't, check if there's a `selection_start` / `selection_end` field you can use instead. Adapt accordingly.

## TASK 6: Comment Toggle + Extra Keyboard Shortcuts

Wire remaining keyboard shortcuts:
```rust
// Ctrl+/ → Toggle line comment
Key::Character(ref c) if c == "/" && ctrl => {
    if let Some(ed) = state.tab_manager.active_editor_mut() {
        let line = ed.cursor_line();
        let text = ed.buffer.text();
        crate::comment_toggle::toggle_comment(ed, line);
        ed.rehighlight();
    }
    state.window.request_redraw();
}

// Ctrl+Z → Undo (if undo exists on editor)
Key::Character(ref c) if c == "z" && ctrl && !self.modifiers.shift_key() => {
    if let Some(ed) = state.tab_manager.active_editor_mut() {
        ed.undo(); // check if this exists
        ed.rehighlight();
    }
    state.window.request_redraw();
}

// Ctrl+Shift+Z or Ctrl+Y → Redo
Key::Character(ref c) if (c == "Z" || c == "y") && ctrl => {
    if let Some(ed) = state.tab_manager.active_editor_mut() {
        ed.redo(); // check if this exists
        ed.rehighlight();
    }
    state.window.request_redraw();
}

// Ctrl+S → Save file
Key::Character(ref c) if c == "s" && ctrl => {
    if let Some(tab) = state.tab_manager.tabs.get(state.tab_manager.active) {
        if let Some(ref path) = tab.path {
            if let Some(ed) = state.tab_manager.active_editor() {
                let text = ed.buffer.text();
                if let Err(e) = std::fs::write(path, &text) {
                    tracing::error!("Save failed: {}", e);
                }
            }
        }
    }
    state.window.request_redraw();
}

// Ctrl+Shift+O → Outline panel
Key::Character(ref c) if c == "O" && ctrl && self.modifiers.shift_key() => {
    self.outline_panel.visible = !self.outline_panel.visible;
    if self.outline_panel.visible {
        if let Some(ed) = state.tab_manager.active_editor() {
            self.outline_panel.refresh(&ed.buffer.text());
        }
    }
    state.window.request_redraw();
}
```

**Read `comment_toggle.rs` and `editor.rs` FIRST** to verify exact function signatures.

## TASK 7: Welcome Screen

When no file is open (tabmanager empty), render a welcome screen:

```rust
// In the render loop, when no active editor:
if state.tab_manager.tabs.is_empty() {
    let welcome_text = "
    ╭────────────────────────────────────────╮
    │                                        │
    │            🔥 FORGE EDITOR             │
    │                                        │
    │    GPU-Accelerated Code Editing         │
    │                                        │
    │    Ctrl+N    New File                   │
    │    Ctrl+O    Open File                  │
    │    Ctrl+P    Quick Open                 │
    │    Ctrl+`    Toggle Terminal            │
    │    Ctrl+,    Settings                   │
    │                                        │
    │    Recent Files:                        │
    │    (none)                               │
    │                                        │
    ╰────────────────────────────────────────╯
    ";

    state.editor_buffer.set_size(
        &mut state.font_system,
        Some(state.layout.editor.width),
        Some(state.layout.editor.height),
    );
    state.editor_buffer.set_text(
        &mut state.font_system,
        welcome_text,
        Attrs::new().family(Family::Monospace)
            .color(Application::theme_glyph_color(self.theme.colors.foreground)),
        Shaping::Advanced,
    );
}
```

## TASK 8: Git Gutter Marks in Editor Gutter

Wire `git_gutter` marks into the gutter rendering:

```rust
// In gutter rendering section, after drawing line numbers:
if let Some(editor) = state.tab_manager.active_editor() {
    // Get current file's original content (from git HEAD)
    // For now, compute based on empty string as baseline
    let marks = crate::git_gutter::compute_gutter_marks("", &editor.buffer.text());
    for mark in &marks {
        if mark.line >= scroll_top && mark.line < scroll_top + vis_lines {
            let y = state.layout.gutter.y + ((mark.line - scroll_top) as f32 * LayoutConstants::LINE_HEIGHT);
            let color = crate::git_gutter::gutter_color(&mark.kind);
            let color_f32 = [color[0] as f32 / 255.0, color[1] as f32 / 255.0, color[2] as f32 / 255.0, 0.8];
            // Draw a 3px colored bar at left edge of gutter
            state.render_batch.push(RectPrimitive {
                x: state.layout.gutter.x,
                y,
                width: 3.0,
                height: LayoutConstants::LINE_HEIGHT,
                color: color_f32,
                border_radius: 0.0,
            });
        }
    }
}
```

**Read `git_gutter.rs` FIRST** to verify `compute_gutter_marks` signature and `gutter_color` function.

---

## FINAL VERIFICATION

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test --workspace
cargo check --workspace
```

ALL FOUR must exit 0.

After this session, the editor should have:
- ✅ Theme-aware rendering (no hardcoded colors)
- ✅ Settings panel (Ctrl+,)
- ✅ Notifications (toast popups)
- ✅ Context menu (right-click)
- ✅ Selection highlighting
- ✅ Comment toggle (Ctrl+/)
- ✅ Save (Ctrl+S)
- ✅ Undo/Redo
- ✅ Welcome screen
- ✅ Git gutter marks
- ✅ Outline panel (Ctrl+Shift+O)

**Read every module file BEFORE writing code that references it. Match reality, not documentation.**
