# EPIC: Bring Forge IDE to Life — Wire ALL Orphaned Systems

## CRITICAL CONTEXT

Forge has **80 source files** in `crates/forge-app/src/` but only ~15 are wired to `application.rs`.
The rest are **orphaned data models with working tests but ZERO rendering or input wiring**.

This epic will wire EVERY orphaned system into `application.rs` so the IDE becomes **fully interactive**.

**MANDATORY RULE: `cargo check -p forge-app` after EVERY change. Never accumulate changes.**

---

## FILE INVENTORY — What's Wired vs Orphaned

### ✅ Currently Wired (DO NOT BREAK)
- `application.rs` — Main loop, rendering, input
- `editor.rs` — Text buffer, cursor, typing
- `tab_manager.rs` — Tab open/close/switch
- `file_explorer.rs` — File tree data
- `file_tree_ui.rs` — File tree rendering
- `ui.rs` — Layout zones, constants
- `gutter.rs` — Line numbers
- `breadcrumb.rs` — Breadcrumb bar
- `activity_bar.rs` — Activity bar icons
- `cursor.rs` — Cursor blink rendering
- `rect_renderer.rs` — Rectangle GPU rendering
- `gpu.rs` — wgpu setup
- `scrollbar.rs` — Scrollbar rendering

### 🔴 ORPHANED (Must Wire)
| File | Lines | What It Does | Has Tests |
|------|-------|-------------|-----------|
| `title_bar.rs` | 541 | Menu bar with File/Edit/View/Go/Run/Terminal/Help menus, hit testing, open/close/toggle | ✅ Yes |
| `context_menu.rs` | 133 | Right-click context menu with Cut/Copy/Paste/Go to Def/Rename | ✅ Yes |
| `command_palette.rs` | 305 | Command palette (Ctrl+Shift+P) with fuzzy filter, Commands & Files modes | ✅ Yes |
| `find_bar.rs` | 272 | Find/Replace with regex, case-sensitive, whole-word matching | ✅ Yes |
| `bottom_panel.rs` | 91 | Bottom panel with Problems/Output/Terminal/DebugConsole tabs | ✅ Yes |
| `terminal_ui.rs` | 55 | Terminal UI rendering with cursor | No |
| `git_panel.rs` | 162 | Git panel with stage/unstage/commit via libgit2 | No |
| `debug_ui.rs` | 118 | Debug panel with breakpoints, stack frames, DAP client | No |
| `go_to_line.rs` | ~50 | Go to line dialog (Ctrl+G) | Unknown |
| `go_to_def.rs` | ~50 | Go to definition (F12) | Unknown |
| `hover_info.rs` | ~50 | Hover tooltips from LSP | Unknown |
| `autocomplete.rs` | ~100 | Autocomplete dropdown | Unknown |
| `minimap.rs` | ~50 | Code minimap sidebar | Unknown |
| `code_folding.rs` | ~50 | Code folding regions | Unknown |
| `bracket_match.rs` | ~50 | Bracket matching highlights | Unknown |
| `indent_guides.rs` | ~50 | Indentation guide lines | Unknown |
| `file_picker.rs` | ~50 | OS file open dialog | Unknown |
| `diff_view.rs` | ~50 | Git diff viewer | Unknown |
| `markdown_preview.rs`| ~50 | Markdown preview panel | Unknown |
| `drag_drop.rs` | ~50 | Tab drag-and-drop | Unknown |

---

## PHASE 1: Menu System (Makes File/Edit/etc ALIVE)

### What to Do

The `TitleBar` struct in `title_bar.rs` already has:
- `create_defaults()` — populates all 8 menus with items
- `hit_test_menu_label(x: f32) -> Option<usize>` — detects which menu label was clicked
- `open_menu(idx)` / `close_menu()` / `toggle_menu(idx)`
- `handle_click(item_idx) -> Option<String>` — returns action string

**You must wire these into `application.rs`:**

#### Step 1.1: Add TitleBar to AppState

In `application.rs`, find the `AppState` struct (or equivalent state struct). Add:

```rust
use crate::title_bar::TitleBar;
// In the state struct:
title_bar: TitleBar,
```

Initialize in `new()`:
```rust
let mut title_bar = TitleBar::new();
title_bar.create_defaults();
```

**`cargo check -p forge-app` — must pass.**

#### Step 1.2: Render Menu Dropdown

In the `render()` function of `application.rs`, AFTER rendering the title bar text labels, add:

```rust
if state.title_bar.is_open() {
    // Find the active menu
    if let Some(active_idx) = state.title_bar.active_menu_idx {
        let menu = &state.title_bar.menus[active_idx];
        // Calculate dropdown position based on menu label position
        // Each label is roughly 70px wide, starting at x=0
        let menu_x = (active_idx as f32) * 70.0;
        let menu_y = LayoutConstants::TITLE_BAR_HEIGHT; // Below title bar
        
        // Render dropdown background rect
        let dropdown_width = 250.0;
        let item_height = 24.0;
        let dropdown_height = menu.items.len() as f32 * item_height;
        
        // Background rect (dark)
        rects.push(Rect {
            x: menu_x,
            y: menu_y,
            width: dropdown_width,
            height: dropdown_height,
            color: [0.17, 0.17, 0.17, 1.0], // #2b2b2b
        });
        
        // Render each menu item as text using glyphon
        for (i, item) in menu.items.iter().enumerate() {
            let item_y = menu_y + (i as f32 * item_height);
            // Use font_system to render item.label at (menu_x + 8, item_y)
            // If item.separator, render a thin horizontal line instead
            // If item has shortcut, render shortcut right-aligned
        }
    }
}
```

**IMPORTANT: Use the SAME glyphon text rendering pattern used for breadcrumb text or file explorer text. Study how those render text and replicate the pattern.**

**`cargo check -p forge-app` — must pass.**

#### Step 1.3: Wire Menu Click Handler

In `handle_input()`, find the section that handles `WindowEvent::MouseInput` with `ElementState::Pressed`. Add a check BEFORE other zone checks (menus take priority):

```rust
// 1. If a dropdown is open, check if click is inside it
if state.title_bar.is_open() {
    if let Some(active_idx) = state.title_bar.active_menu_idx {
        let menu = &state.title_bar.menus[active_idx];
        let menu_x = (active_idx as f32) * 70.0;
        let menu_y = LayoutConstants::TITLE_BAR_HEIGHT;
        let item_height = 24.0;
        let dropdown_width = 250.0;
        let dropdown_height = menu.items.len() as f32 * item_height;
        
        if mx >= menu_x && mx < menu_x + dropdown_width
            && my >= menu_y && my < menu_y + dropdown_height
        {
            let item_idx = ((my - menu_y) / item_height) as usize;
            if let Some(action) = state.title_bar.handle_click(item_idx) {
                // Execute the action
                execute_action(&action, state);
            }
            state.title_bar.close_menu();
            state.window.request_redraw();
            return;
        } else {
            // Clicked outside dropdown — close it
            state.title_bar.close_menu();
        }
    }
}

// 2. Check if click is on a menu label in the title bar
if my < LayoutConstants::TITLE_BAR_HEIGHT {
    if let Some(menu_idx) = state.title_bar.hit_test_menu_label(mx) {
        state.title_bar.toggle_menu(menu_idx);
        state.window.request_redraw();
        return;
    }
}
```

#### Step 1.4: Create Action Dispatcher

Create a function that maps action strings to actual operations:

```rust
fn execute_action(action: &str, state: &mut AppState) {
    match action {
        "file.new" => { state.tab_manager.open_scratch(); }
        "file.open" => {
            // Use rfd::FileDialog or native-dialog crate
            if let Some(path) = rfd::FileDialog::new().pick_file() {
                let content = std::fs::read_to_string(&path).unwrap_or_default();
                state.tab_manager.open_file(path.display().to_string(), content);
            }
        }
        "file.save" => {
            if let Some(editor) = state.tab_manager.active_editor() {
                if let Some(path) = &editor.file_path {
                    let _ = std::fs::write(path, editor.buffer.text());
                }
            }
        }
        "file.save_as" => {
            if let Some(path) = rfd::FileDialog::new().save_file() {
                if let Some(editor) = state.tab_manager.active_editor() {
                    let _ = std::fs::write(&path, editor.buffer.text());
                }
            }
        }
        "edit.undo" => {
            if let Some(ed) = state.tab_manager.active_editor_mut() { ed.undo(); }
        }
        "edit.redo" => {
            if let Some(ed) = state.tab_manager.active_editor_mut() { ed.redo(); }
        }
        "edit.cut" => {
            if let Some(ed) = state.tab_manager.active_editor_mut() { ed.cut(); }
        }
        "edit.copy" => {
            if let Some(ed) = state.tab_manager.active_editor_mut() { ed.copy(); }
        }
        "edit.paste" => {
            if let Some(ed) = state.tab_manager.active_editor_mut() { ed.paste(); }
        }
        "edit.select_all" => {
            if let Some(ed) = state.tab_manager.active_editor_mut() { ed.select_all(); }
        }
        "edit.find" => { state.find_bar.open(); }
        "edit.replace" => { state.find_bar.open(); /* TODO: switch to replace mode */ }
        "view.command_palette" => { state.command_palette.open(crate::command_palette::PaletteMode::Commands); }
        "view.explorer" => { state.sidebar_open = !state.sidebar_open; }
        "view.terminal" => { state.bottom_panel.set_tab(crate::bottom_panel::PanelTab::Terminal); }
        "view.problems" => { state.bottom_panel.set_tab(crate::bottom_panel::PanelTab::Problems); }
        "view.output" => { state.bottom_panel.set_tab(crate::bottom_panel::PanelTab::Output); }
        "go.goto_line" => { /* open go_to_line dialog */ }
        "go.goto_file" => { state.command_palette.open(crate::command_palette::PaletteMode::Files); }
        "run.start_debug" => { /* F5 */ }
        "terminal.new" => { state.bottom_panel.set_tab(crate::bottom_panel::PanelTab::Terminal); }
        _ => { tracing::warn!("Unknown action: {}", action); }
    }
}
```

**NOTE:** Check if `rfd` crate is in `Cargo.toml`. If not, add it:
```toml
rfd = "0.15"
```
If `rfd` causes compilation issues on the target platform, use `native-dialog` instead, or stub the file picker.

**`cargo check -p forge-app` — must pass.**

#### Step 1.5: Menu Hover Behavior

In `handle_input()`, find the `WindowEvent::CursorMoved` handler. Add:

```rust
// If a menu is already open and cursor moves over a different label, switch to that menu
if state.title_bar.is_open() && my < LayoutConstants::TITLE_BAR_HEIGHT {
    if let Some(menu_idx) = state.title_bar.hit_test_menu_label(mx) {
        state.title_bar.open_menu(menu_idx);
        state.window.request_redraw();
    }
}
```

**`cargo check -p forge-app` — must pass.**

#### Step 1.6: Escape Closes Menu

In the `WindowEvent::KeyboardInput` handler, add near the top:

```rust
// Escape closes any open menu/dropdown/palette
if key == Key::Named(NamedKey::Escape) {
    if state.title_bar.is_open() {
        state.title_bar.close_menu();
        state.window.request_redraw();
        return;
    }
    if state.command_palette.visible {
        state.command_palette.close();
        state.window.request_redraw();
        return;
    }
    if state.find_bar.visible {
        state.find_bar.close();
        state.window.request_redraw();
        return;
    }
    if state.context_menu.visible {
        state.context_menu.hide();
        state.window.request_redraw();
        return;
    }
}
```

**`cargo check -p forge-app` — must pass.**

**Verification:** Take screenshot with `cargo run -p forge-app -- --screenshot screenshot_phase1.png`. Click on "File" in the menu bar should show dropdown items.

---

## PHASE 2: Context Menu (Right-Click Menu)

### What to Do

`context_menu.rs` already has `ContextMenu` with `show()`, `hide()`, `handle_click()`, and `editor_context()`.

#### Step 2.1: Add ContextMenu to AppState

```rust
use crate::context_menu::ContextMenu;
// In state struct:
context_menu: ContextMenu,
```

Initialize: `context_menu: ContextMenu::new(),`

**`cargo check -p forge-app` — must pass.**

#### Step 2.2: Wire Right-Click

In `handle_input()`, find `MouseButton::Right` handling (or add it):

```rust
MouseButton::Right => {
    if state.layout.editor.contains(mx, my) {
        state.context_menu.show(mx, my, ContextMenu::editor_context());
        state.window.request_redraw();
    }
}
```

**`cargo check -p forge-app` — must pass.**

#### Step 2.3: Render Context Menu

In `render()`, AFTER all other rendering (context menu renders on top of everything):

```rust
if state.context_menu.visible {
    let cm = &state.context_menu;
    let item_h = 24.0;
    let width = 220.0;
    let height = cm.items.len() as f32 * item_h;
    
    // Shadow rect
    rects.push(Rect {
        x: cm.x + 2.0, y: cm.y + 2.0,
        width, height,
        color: [0.0, 0.0, 0.0, 0.3],
    });
    
    // Background
    rects.push(Rect {
        x: cm.x, y: cm.y,
        width, height,
        color: [0.17, 0.17, 0.17, 1.0],
    });
    
    // Each item text via glyphon
    for (i, item) in cm.items.iter().enumerate() {
        if item.separator {
            // Thin horizontal line
            rects.push(Rect {
                x: cm.x + 8.0,
                y: cm.y + (i as f32 * item_h) + item_h / 2.0,
                width: width - 16.0,
                height: 1.0,
                color: [0.3, 0.3, 0.3, 1.0],
            });
        }
        // Render item.label text at (cm.x + 24, cm.y + i * item_h)
        // Render item.shortcut right-aligned if present
    }
}
```

#### Step 2.4: Wire Context Menu Click

In the left-click handler, BEFORE other checks:

```rust
if state.context_menu.visible {
    let cm = &state.context_menu;
    let item_h = 24.0_f32;
    let width = 220.0_f32;
    let height = cm.items.len() as f32 * item_h;
    
    if mx >= cm.x && mx < cm.x + width && my >= cm.y && my < cm.y + height {
        let idx = ((my - cm.y) / item_h) as usize;
        if let Some(action) = state.context_menu.handle_click(idx) {
            execute_action(&action, state);
        }
    }
    state.context_menu.hide();
    state.window.request_redraw();
    return;
}
```

**`cargo check -p forge-app` — must pass.**

---

## PHASE 3: Command Palette (Ctrl+Shift+P)

### What to Do

`command_palette.rs` has `CommandPalette` with:
- `open(mode)` / `close()` — toggle visibility
- `type_char(c)` / `backspace()` — input handling
- `filter()` — fuzzy filtering
- `select_command(idx)` / `select_file(idx)` — selection
- `register_defaults()` — registers ~50 default commands

#### Step 3.1: Add CommandPalette to AppState

```rust
use crate::command_palette::{CommandPalette, PaletteMode};
// In state struct:
command_palette: CommandPalette,
```

Initialize:
```rust
let mut command_palette = CommandPalette::new();
command_palette.register_defaults();
```

**`cargo check -p forge-app` — must pass.**

#### Step 3.2: Wire Ctrl+Shift+P and F1

In the keyboard handler:

```rust
// Ctrl+Shift+P or F1 — Command Palette
if (modifiers.control_key() && modifiers.shift_key() && key == Key::Character("p".into()))
    || key == Key::Named(NamedKey::F1)
{
    if state.command_palette.visible {
        state.command_palette.close();
    } else {
        state.command_palette.open(PaletteMode::Commands);
    }
    state.window.request_redraw();
    return;
}

// Ctrl+P — Quick Open (file mode)
if modifiers.control_key() && !modifiers.shift_key() && key == Key::Character("p".into()) {
    state.command_palette.open(PaletteMode::Files);
    // Populate files list from file explorer
    if let Some(ref fe) = state.file_explorer {
        let files: Vec<String> = fe.all_files().iter().map(|f| f.path.clone()).collect();
        state.command_palette.set_files(files);
    }
    state.window.request_redraw();
    return;
}
```

#### Step 3.3: Render Command Palette Overlay

The palette renders as a centered overlay at the top of the window (like VS Code):

```rust
if state.command_palette.visible {
    let (win_w, _win_h) = state.window_size;
    let palette_w = 600.0_f32.min(win_w * 0.6);
    let palette_x = (win_w - palette_w) / 2.0;
    let palette_y = LayoutConstants::TITLE_BAR_HEIGHT + 10.0;
    let input_h = 32.0;
    let item_h = 24.0;
    
    let items_to_show = if state.command_palette.mode == PaletteMode::Commands {
        state.command_palette.filtered_commands.len().min(12)
    } else {
        state.command_palette.filtered_files.len().min(12)
    };
    let total_h = input_h + (items_to_show as f32 * item_h);
    
    // Semi-transparent overlay behind palette
    // (optional, VS Code uses it)
    
    // Palette background
    rects.push(Rect {
        x: palette_x, y: palette_y,
        width: palette_w, height: total_h,
        color: [0.15, 0.15, 0.15, 1.0],
    });
    
    // Input field background
    rects.push(Rect {
        x: palette_x + 4.0, y: palette_y + 4.0,
        width: palette_w - 8.0, height: input_h - 8.0,
        color: [0.2, 0.2, 0.2, 1.0],
    });
    
    // Render input text (state.command_palette.query) via glyphon
    // Render prefix ">" for commands mode
    
    // Render each filtered item
    // Highlight the selected item with a different background color
}
```

#### Step 3.4: Command Palette Input Routing

When the command palette is visible, keyboard input should go to IT, not the editor:

```rust
// At the TOP of KeyboardInput handler:
if state.command_palette.visible {
    match &key {
        Key::Named(NamedKey::Escape) => {
            state.command_palette.close();
            state.window.request_redraw();
            return;
        }
        Key::Named(NamedKey::Enter) => {
            // Execute selected command/open selected file
            if state.command_palette.mode == PaletteMode::Commands {
                if let Some(idx) = state.command_palette.selected_index {
                    if let Some(cmd) = state.command_palette.select_command(idx) {
                        let action = cmd.action.clone();
                        state.command_palette.close();
                        execute_action(&action, state);
                    }
                }
            } else {
                if let Some(idx) = state.command_palette.selected_index {
                    if let Some(file) = state.command_palette.select_file(idx) {
                        let path = file.clone();
                        state.command_palette.close();
                        let content = std::fs::read_to_string(&path).unwrap_or_default();
                        state.tab_manager.open_file(path, content);
                    }
                }
            }
            state.window.request_redraw();
            return;
        }
        Key::Named(NamedKey::ArrowDown) => {
            // Move selection down
            if let Some(ref mut idx) = state.command_palette.selected_index {
                *idx += 1;
            } else {
                state.command_palette.selected_index = Some(0);
            }
            state.window.request_redraw();
            return;
        }
        Key::Named(NamedKey::ArrowUp) => {
            // Move selection up
            if let Some(ref mut idx) = state.command_palette.selected_index {
                if *idx > 0 { *idx -= 1; }
            }
            state.window.request_redraw();
            return;
        }
        Key::Named(NamedKey::Backspace) => {
            state.command_palette.backspace();
            state.command_palette.filter();
            state.window.request_redraw();
            return;
        }
        Key::Character(c) => {
            for ch in c.chars() {
                state.command_palette.type_char(ch);
            }
            state.command_palette.filter();
            state.window.request_redraw();
            return;
        }
        _ => { return; }
    }
}
```

**`cargo check -p forge-app` — must pass.**

---

## PHASE 4: Find Bar (Ctrl+F)

### What to Do

`find_bar.rs` has `FindBar` with search (regex, case-sensitive, whole-word), next/prev match navigation.

#### Step 4.1: Add FindBar to AppState

```rust
use crate::find_bar::FindBar;
// In state struct:
find_bar: FindBar,
```

**`cargo check -p forge-app` — must pass.**

#### Step 4.2: Wire Ctrl+F

```rust
if modifiers.control_key() && key == Key::Character("f".into()) {
    if state.find_bar.visible {
        state.find_bar.close();
    } else {
        state.find_bar.open();
    }
    state.window.request_redraw();
    return;
}
```

#### Step 4.3: Render Find Bar

Render at top-right of editor area (like VS Code):

```rust
if state.find_bar.visible {
    let editor_zone = &state.layout.editor;
    let bar_w = 350.0;
    let bar_h = 32.0;
    let bar_x = editor_zone.x + editor_zone.width - bar_w - 16.0;
    let bar_y = editor_zone.y;
    
    // Background
    rects.push(Rect {
        x: bar_x, y: bar_y,
        width: bar_w, height: bar_h,
        color: [0.15, 0.15, 0.18, 1.0],
    });
    
    // Input field
    rects.push(Rect {
        x: bar_x + 4.0, y: bar_y + 4.0,
        width: bar_w - 100.0, height: bar_h - 8.0,
        color: [0.2, 0.2, 0.22, 1.0],
    });
    
    // Render query text via glyphon
    // Render match count text: "N of M"
    // Render up/down arrows for next/prev match
}
```

#### Step 4.4: Find Bar Input Routing

When find bar is visible, keyboard input goes to it:

```rust
if state.find_bar.visible {
    match &key {
        Key::Named(NamedKey::Escape) => {
            state.find_bar.close();
            state.window.request_redraw();
            return;
        }
        Key::Named(NamedKey::Enter) => {
            // Next match
            state.find_bar.next_match();
            // Scroll editor to match location
            if let Some(m) = state.find_bar.matches.get(state.find_bar.current_match.unwrap_or(0)) {
                if let Some(ed) = state.tab_manager.active_editor_mut() {
                    ed.goto_line(m.line);
                }
            }
            state.window.request_redraw();
            return;
        }
        Key::Named(NamedKey::Backspace) => {
            state.find_bar.query.pop();
            // Re-search
            if let Some(ed) = state.tab_manager.active_editor() {
                let text = ed.buffer.text();
                state.find_bar.search(&text, &state.find_bar.query.clone());
            }
            state.window.request_redraw();
            return;
        }
        Key::Character(c) => {
            state.find_bar.query.push_str(c);
            // Re-search
            if let Some(ed) = state.tab_manager.active_editor() {
                let text = ed.buffer.text();
                let query = state.find_bar.query.clone();
                state.find_bar.search(&text, &query);
            }
            state.window.request_redraw();
            return;
        }
        _ => { return; }
    }
}
```

#### Step 4.5: Highlight Matches in Editor

In the editor rendering section, add match highlighting:

```rust
if state.find_bar.visible && !state.find_bar.matches.is_empty() {
    for m in &state.find_bar.matches {
        let y = editor_zone.y + ((m.line as f32 - scroll_top as f32) * LINE_HEIGHT);
        if y >= editor_zone.y && y < editor_zone.y + editor_zone.height {
            let x = editor_zone.x + GUTTER_WIDTH + (m.start_col as f32 * CHAR_WIDTH);
            let w = ((m.end_col - m.start_col) as f32) * CHAR_WIDTH;
            rects.push(Rect {
                x, y, width: w, height: LINE_HEIGHT,
                color: [0.4, 0.35, 0.0, 0.4], // Yellow-ish highlight
            });
        }
    }
}
```

**`cargo check -p forge-app` — must pass.**

---

## PHASE 5: Bottom Panel (Terminal/Problems/Output)

### What to Do

`bottom_panel.rs` has `BottomPanel` with `toggle()`, `set_tab()`, `resize()`.
`terminal_ui.rs` has `TerminalUi` with `render_rects()`.

#### Step 5.1: Add BottomPanel to AppState

```rust
use crate::bottom_panel::{BottomPanel, PanelTab};
// In state struct:
bottom_panel: BottomPanel,
```

**`cargo check -p forge-app` — must pass.**

#### Step 5.2: Wire Ctrl+` (Toggle Terminal)

```rust
if modifiers.control_key() && key == Key::Character("`".into()) {
    state.bottom_panel.set_tab(PanelTab::Terminal);
    state.window.request_redraw();
    return;
}

// Ctrl+Shift+M — Problems
if modifiers.control_key() && modifiers.shift_key() && key == Key::Character("m".into()) {
    state.bottom_panel.set_tab(PanelTab::Problems);
    state.window.request_redraw();
    return;
}
```

#### Step 5.3: Adjust Layout for Bottom Panel

In `LayoutZones::compute()`, when `bottom_panel.visible`:

```rust
let bottom_panel_h = if bottom_panel.visible { bottom_panel.height } else { 0.0 };
let editor_h = total_h - editor_y - status_bar_h - bottom_panel_h;
// Add bottom_panel Zone
let bottom_panel_zone = if bottom_panel.visible {
    Some(Zone::new(content_x, editor_y + editor_h, content_w, bottom_panel_h))
} else {
    None
};
```

#### Step 5.4: Render Bottom Panel

```rust
if state.bottom_panel.visible {
    if let Some(ref panel_zone) = state.layout.bottom_panel {
        // Tab bar at top of panel
        let tabs = ["PROBLEMS", "OUTPUT", "TERMINAL", "DEBUG CONSOLE"];
        let tab_h = 28.0;
        
        // Panel background
        rects.push(Rect {
            x: panel_zone.x, y: panel_zone.y,
            width: panel_zone.width, height: panel_zone.height,
            color: [0.1, 0.1, 0.1, 1.0],
        });
        
        // Tab bar
        for (i, tab_name) in tabs.iter().enumerate() {
            let tab_x = panel_zone.x + (i as f32 * 120.0);
            let is_active = match (i, state.bottom_panel.active_tab) {
                (0, PanelTab::Problems) => true,
                (1, PanelTab::Output) => true,
                (2, PanelTab::Terminal) => true,
                (3, PanelTab::DebugConsole) => true,
                _ => false,
            };
            if is_active {
                // Active tab underline
                rects.push(Rect {
                    x: tab_x, y: panel_zone.y + tab_h - 2.0,
                    width: 100.0, height: 2.0,
                    color: [0.0, 0.478, 0.8, 1.0], // Blue
                });
            }
            // Render tab_name text via glyphon
        }
        
        // Panel content area
        match state.bottom_panel.active_tab {
            PanelTab::Terminal => {
                // Render terminal output from forge-terminal crate
                // Use TerminalUi::render_rects() if terminal exists
            }
            PanelTab::Problems => {
                // Render diagnostics from LSP
            }
            PanelTab::Output => {
                // Render build/task output
            }
            PanelTab::DebugConsole => {
                // Render debug output
            }
        }
    }
}
```

#### Step 5.5: Bottom Panel Click Handler

In `handle_input()`:

```rust
if state.bottom_panel.visible {
    if let Some(ref panel_zone) = state.layout.bottom_panel {
        if panel_zone.contains(mx, my) {
            let rel_y = my - panel_zone.y;
            if rel_y < 28.0 {
                // Click on tab bar
                let tab_idx = ((mx - panel_zone.x) / 120.0) as usize;
                match tab_idx {
                    0 => state.bottom_panel.set_tab(PanelTab::Problems),
                    1 => state.bottom_panel.set_tab(PanelTab::Output),
                    2 => state.bottom_panel.set_tab(PanelTab::Terminal),
                    3 => state.bottom_panel.set_tab(PanelTab::DebugConsole),
                    _ => {}
                }
                state.window.request_redraw();
                return;
            }
            // If terminal tab is active, forward input to terminal
        }
    }
}
```

**`cargo check -p forge-app` — must pass.**

---

## PHASE 6: Git Panel (Source Control Sidebar)

### What to Do

`git_panel.rs` has `GitPanel` with `refresh()`, `stage_file()`, `unstage_file()`, `commit()` via libgit2.

#### Step 6.1: Add to AppState

```rust
use crate::git_panel::GitPanel;
// In state struct:
git_panel: GitPanel,
```

#### Step 6.2: Wire Activity Bar Source Control Icon

The activity bar already has a Source Control icon. When clicked, set `sidebar_mode` to `SourceControl`:

```rust
// In SidebarMode enum (ui.rs):
SourceControl,
```

When sidebar_mode is SourceControl, render git panel content:

```rust
if state.sidebar_mode == SidebarMode::SourceControl {
    // Refresh git status
    if let Some(ref dir) = state.current_directory {
        let _ = state.git_panel.refresh(std::path::Path::new(dir));
    }
    
    // Render "SOURCE CONTROL" header
    // Render staged files section
    // Render unstaged files section
    // Each file shows: status icon (M/A/D/U) + filename
    // Clicking a file opens diff view
}
```

#### Step 6.3: Git Actions

- Click on file → stage/unstage toggle
- Commit message input at top of panel
- Enter in commit input → `git_panel.commit(message)`

**`cargo check -p forge-app` — must pass.**

---

## PHASE 7: Remaining Keyboard Shortcuts

Wire ALL missing shortcuts. Check each one:

```rust
// FILE OPERATIONS
Ctrl+N   → state.tab_manager.open_scratch()
Ctrl+O   → file picker dialog → open file
Ctrl+S   → save active file
Ctrl+Shift+S → save as dialog
Ctrl+W   → close active tab

// EDITING
Ctrl+Z   → undo
Ctrl+Y   → redo (or Ctrl+Shift+Z)
Ctrl+C   → copy
Ctrl+X   → cut
Ctrl+V   → paste
Ctrl+A   → select all
Ctrl+D   → select word / add next occurrence
Ctrl+/   → toggle line comment

// NAVIGATION
Ctrl+G   → go to line
Ctrl+P   → quick open file
F12      → go to definition
Ctrl+Tab → cycle tabs

// VIEW
Ctrl+B       → toggle sidebar
Ctrl+`       → toggle terminal
Ctrl+Shift+P → command palette
Ctrl+Shift+E → explorer
Ctrl+Shift+G → source control

// DEBUG
F5       → start/continue debugging
F9       → toggle breakpoint
F10      → step over
F11      → step into
```

For each shortcut:
1. Find the keyboard handler section
2. Check if the shortcut is already handled
3. If not, add the handler
4. If handled but disconnected, wire it to the correct function

**`cargo check -p forge-app` — must pass after each shortcut is added.**

---

## PHASE 8: Visual Polish

#### Step 8.1: Match Highlight for Active Line

```rust
// In editor rendering:
let active_line = editor.cursor_line();
let y = editor_zone.y + ((active_line as f32 - scroll_top as f32) * LINE_HEIGHT);
rects.push(Rect {
    x: editor_zone.x,
    y,
    width: editor_zone.width,
    height: LINE_HEIGHT,
    color: [1.0, 1.0, 1.0, 0.04], // Very subtle highlight
});
```

#### Step 8.2: Selection Highlighting

```rust
// Render selection range as blue highlight
if let Some((start, end)) = editor.selection_range() {
    for line in start.line..=end.line {
        let y = editor_zone.y + ((line as f32 - scroll_top as f32) * LINE_HEIGHT);
        let start_col = if line == start.line { start.col } else { 0 };
        let end_col = if line == end.line { end.col } else { 200 }; // line length
        let x = editor_zone.x + GUTTER_WIDTH + (start_col as f32 * CHAR_WIDTH);
        let w = ((end_col - start_col) as f32) * CHAR_WIDTH;
        rects.push(Rect {
            x, y, width: w, height: LINE_HEIGHT,
            color: [0.04, 0.24, 0.57, 0.5], // VS Code blue selection
        });
    }
}
```

#### Step 8.3: Bracket Matching

Wire `bracket_match.rs` to highlight matching brackets:

```rust
use crate::bracket_match::BracketMatcher;
// When cursor is on a bracket, find its match and highlight both
```

#### Step 8.4: Indent Guides

Wire `indent_guides.rs` to render vertical indentation lines:

```rust
use crate::indent_guides::IndentGuides;
// Render thin vertical lines at each indentation level
```

**`cargo check -p forge-app` — must pass.**

---

## PHASE 9: Wire --debug-zones Flag

#### Step 9.1: Add CLI Flag

In `main.rs`, add `--debug-zones` to the argument parser.

#### Step 9.2: Pass to Application

```rust
// In AppState:
debug_zones: bool,
```

#### Step 9.3: Render Zone Borders

At the END of `render()`, if `debug_zones` is true:

```rust
if state.debug_zones {
    let zones = vec![
        (&state.layout.title_bar, "title_bar", [1.0, 0.0, 0.0, 1.0]),
        (&state.layout.activity_bar, "activity_bar", [0.0, 1.0, 0.0, 1.0]),
        (&state.layout.tab_bar, "tab_bar", [0.0, 0.0, 1.0, 1.0]),
        (&state.layout.editor, "editor", [1.0, 1.0, 0.0, 1.0]),
        (&state.layout.breadcrumb_bar, "breadcrumb", [0.0, 1.0, 1.0, 1.0]),
        (&state.layout.gutter, "gutter", [1.0, 0.0, 1.0, 1.0]),
        (&state.layout.status_bar, "status_bar", [1.0, 0.5, 0.0, 1.0]),
    ];
    
    if let Some(ref sb) = state.layout.sidebar {
        // Add sidebar zone
    }
    
    for (zone, _name, color) in &zones {
        // Top border
        rects.push(Rect { x: zone.x, y: zone.y, width: zone.width, height: 2.0, color: *color });
        // Bottom border
        rects.push(Rect { x: zone.x, y: zone.y + zone.height - 2.0, width: zone.width, height: 2.0, color: *color });
        // Left border
        rects.push(Rect { x: zone.x, y: zone.y, width: 2.0, height: zone.height, color: *color });
        // Right border
        rects.push(Rect { x: zone.x + zone.width - 2.0, y: zone.y, width: 2.0, height: zone.height, color: *color });
    }
}
```

**`cargo check -p forge-app` — must pass.**

---

## FINAL VERIFICATION

After ALL phases:

```bash
# 1. Build clean
cargo check -p forge-app 2>&1

# 2. Run tests
cargo test -p forge-app 2>&1

# 3. Screenshot — normal mode
cargo run -p forge-app -- --screenshot screenshot_final.png

# 4. Screenshot — debug zones mode  
cargo run -p forge-app -- --debug-zones --screenshot screenshot_zones.png

# 5. Test that menus open
# (visual verification from screenshot — the menu should not crash)

# 6. No hardcoded panics or unwrap on user data
grep -r "unwrap()" crates/forge-app/src/ | grep -v "test" | grep -v "// safe"
# Review each one — should be .unwrap_or() or if let
```

---

## ANTI-CRASH RULES (SAME AS BEFORE)

1. **`cargo check -p forge-app` after EVERY change.** Period.
2. **Never `unwrap()` on user data.** Use `.unwrap_or()`, `if let`, or `?`.
3. **Never index arrays directly.** Use `.get(idx)` with bounds checking.
4. **Borrow checker:** `state.font_system` conflicts — create local vars before exclusive borrows.
5. **Edit `application.rs` surgically** — it's 3100+ lines. Small targeted edits only.
6. **glyphon:** Always `shape_until_scroll()` after `set_text()`.
7. **wgpu:** Surface texture presented exactly once per frame.
8. **winit:** No blocking I/O in event handlers.
9. **New crates in `Cargo.toml`** need `[workspace.members]` entry if workspace uses resolver 2.
10. **Test screenshot after each phase** — `cargo run -p forge-app -- --screenshot`.
