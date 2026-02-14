# FORGE IDE → REAL VS CODE UI — JULES MASTER PROMPT

> **Project:** `forge` (Rust, wgpu, winit, glyphon)
> **Location:** `c:\Users\osaretin\Downloads\smslycloud-master\forge\`
> **Crate:** `forge-app` (main application crate)
> **Goal:** Make Forge look and behave exactly like VS Code. Then build interactive testing tools.

---

## PHASE 0: ZERO-TRUST CONTEXT LOAD

**Before writing ANY code, you MUST:**

1. Run `cargo check -p forge-app` — confirm it compiles. If not, fix errors first.
2. Run `cargo run -p forge-app -- --screenshot screenshot_before.png` — capture a baseline screenshot.
3. Read these files completely before proceeding:
   - `forge/crates/forge-app/src/ui.rs` — all layout constants and zone computation
   - `forge/crates/forge-app/src/application.rs` — the 3100+ line monolith (rendering, input, init)
   - `forge/crates/forge-app/src/editor.rs` — editor state
   - `forge/crates/forge-app/src/tab_manager.rs` — tab management
4. **Log every assumption** using this format in your working notes:
   ```
   ASSUMPTION: [claim] | VERIFIED: [Yes/No] | HOW: [method]
   ```

---

## PHASE 1: VS CODE LAYOUT — EXACT PIXEL SPECS

### Target Layout (Top-to-Bottom, Left-to-Right)

```
┌──────────────────────────────────────────────────────────────┐
│  Title Bar (30px) — window controls + drag area              │
│  Menu Bar (30px) — File Edit Selection View Go Run Term Help │
├────┬──────────┬──────────────────────────────┬───────────────┤
│ AB │ Sidebar  │ Tab Bar (35px)               │ AI Panel      │
│48px│ 250px    │ Breadcrumb Bar (22px)         │ 350px         │
│    │          ├─────┬───────────────────┬────┤               │
│    │          │Gutter│  Editor Area     │SB  │               │
│    │          │ 60px │  (remaining)     │14px│               │
│    │          │      │                  │    │               │
├────┴──────────┴─────┴───────────────────┴────┴───────────────┤
│  Status Bar (22px) — mode, branch, errors, line/col, lang    │
└──────────────────────────────────────────────────────────────┘
```

### Current Constants in `ui.rs` (Line ~69-87)

```rust
pub const TITLE_BAR_HEIGHT: f32 = 30.0;
pub const ACTIVITY_BAR_WIDTH: f32 = 48.0;
pub const TAB_BAR_HEIGHT: f32 = 35.0;
pub const BREADCRUMB_HEIGHT: f32 = 22.0;
pub const STATUS_BAR_HEIGHT: f32 = 22.0;
pub const GUTTER_WIDTH: f32 = 60.0;
pub const SIDEBAR_WIDTH: f32 = 250.0;
pub const SCROLLBAR_WIDTH: f32 = 14.0;
pub const TAB_WIDTH: f32 = 160.0;
pub const AI_PANEL_WIDTH: f32 = 350.0;
pub const LINE_HEIGHT: f32 = 22.0;
pub const CHAR_WIDTH: f32 = 9.0;
pub const FONT_SIZE: f32 = 16.0;
pub const SMALL_FONT_SIZE: f32 = 13.0;
```

> These are already VS Code-accurate. **Do NOT change them** unless you have a measured reason.

### Zone Computation

`LayoutZones::compute()` in `ui.rs` (line ~141-245) calculates all zones. The logic is correct but verify:
- `activity_bar` starts at (0, title_bar_h) — ✅
- `sidebar` starts at (activity_bar_width, title_bar_h) — ✅
- `editor` starts at (content_x + gutter_w, editor_y) — **Verify this**
- Status bar is at (0, window_height - STATUS_BAR_HEIGHT) — ✅

---

## PHASE 2: INPUT HANDLING — COMPLETE FIX

### Known Bugs (Verified)

1. **Mouse click offset in file explorer:**
   - Header text `"  EXPLORER\n\n"` = 2 lines at LINE_HEIGHT = 44px
   - Click handler at `application.rs` line ~2240 uses `header_offset = 44.0` (already fixed)
   - `FileTreeUi::render_rects()` in `file_tree_ui.rs` accepts `header_lines` parameter (already fixed)
   - **VERIFY** that the render_rects call site in application.rs passes `2` for header_lines. If there is no call site, wire it up.

2. **Editor cursor click:**
   - Handler at `application.rs` line ~2194
   - Uses `rel_x / CHAR_WIDTH` for column — correct for monospace only
   - Uses `rel_y / LINE_HEIGHT` for line — correct
   - **MUST account for scroll offset** (already does via `scroll_top`)
   - **VERIFY** the calculation is accurate by testing with the screenshot tool

3. **Keyboard input requires a tab:**
   - `active_editor_mut()` returns `None` if no tabs
   - `open_scratch()` is called at init (already fixed in `tab_manager.rs`)
   - **VERIFY** it works by checking the screenshot shows a Welcome tab

### What to Fix

For **every** `contains(mx, my)` check in `handle_input()` (line ~2049-2325):
1. Print-trace or debug-log the zone boundaries vs mouse position
2. Verify the zone.x, zone.y, zone.width, zone.height match what's rendered
3. Fix any misalignment between render positions and click regions

**Critical zones to audit:**
| Zone | Contains check | Render location |
|------|---------------|-----------------|
| activity_bar | line ~2118 | `render_rects()` in activity_bar.rs |
| tab_bar | line ~2169 | `render_rects()` in tab_bar.rs |
| gutter | line ~2171 | `render_rects()` in gutter.rs |
| scrollbar_v | line ~2187 | scrollbar.rs |
| editor | line ~2194 | glyphon text rendering |
| sidebar (file explorer) | line ~2233 | text buffer + file_tree_ui.rs |

---

## PHASE 3: VS CODE VISUAL FIDELITY

### Color Scheme (Dark+ Theme)

The theme system is in `forge-theme` crate. The `ThemeMap` in `forge-theme/src/lib.rs` provides color lookups.

**VS Code Dark+ exact colors:**

| Element | VS Code Color | RGBA |
|---------|--------------|------|
| Title Bar BG | #3c3c3c | [0.235, 0.235, 0.235, 1.0] |
| Menu Bar BG | #3c3c3c | same |
| Activity Bar BG | #333333 | [0.2, 0.2, 0.2, 1.0] |
| Sidebar BG | #252526 | [0.145, 0.145, 0.149, 1.0] |
| Editor BG | #1e1e1e | [0.118, 0.118, 0.118, 1.0] |
| Status Bar BG | #007acc | [0.0, 0.478, 0.8, 1.0] |
| Tab Active BG | #1e1e1e | [0.118, 0.118, 0.118, 1.0] |
| Tab Inactive BG | #2d2d2d | [0.176, 0.176, 0.176, 1.0] |
| Text FG | #cccccc | [0.8, 0.8, 0.8, 1.0] |
| Text Dim | #858585 | [0.522, 0.522, 0.522, 1.0] |
| Line Number FG | #858585 | same |
| Active Line Number | #c6c6c6 | [0.776, 0.776, 0.776, 1.0] |
| Selection BG | #264f78 | [0.149, 0.31, 0.471, 0.5] |
| Cursor | #aeafad | [0.682, 0.686, 0.678, 1.0] |

**Update `colors` struct** in `ui.rs` (line ~32-63) and the `forge-theme` default theme to match.

### Render Pipeline

Rendering is in `Application::render()` (application.rs, starts around line 595). It:
1. Clears render batch
2. Builds rects for each zone (tab bar, activity bar, gutter, etc.)
3. Builds text buffers (sidebar, breadcrumb, editor, status bar, etc.)
4. Submits to `RectRenderer` + `glyphon` TextRenderer

**Key render sections:**
- Zone background rects: line ~600-740
- Editor text: line ~740-900  
- Sidebar text: line ~1161-1310
- Status bar text: line ~1050-1130
- Dynamic text buffers (overlays): line ~1354-1760
- GPU submit: line ~1850-2030

---

## PHASE 4: STARTUP BEHAVIOR (VS CODE PARITY)

### Current State
- `sidebar_open: false` at init ✅
- `open_scratch()` creates Welcome tab ✅
- No auto file scan ✅ (lazy loads on Explorer click)
- Breadcrumb empty for untitled ✅

### What VS Code Actually Shows on Startup

1. **Welcome Tab content** — not blank. Show:
   ```
   Forge IDE
   
   Start
     New File          (Ctrl+N)
     Open File...      (Ctrl+O)
     Open Folder...    (Ctrl+K Ctrl+O)
   
   Recent
     (no recent files)
   
   Help  
     Show All Commands  (Ctrl+Shift+P / F1)
     Terminal            (Ctrl+`)
   ```

2. **Render this as editor text** in the Welcome tab's buffer. In `tab_manager.rs`, `open_scratch()` creates `Editor::new()` with empty buffer. Instead, pre-populate the buffer with welcome content.

3. **Sidebar stays closed** until user clicks Explorer icon — ✅ already

---

## PHASE 5: FUNCTIONAL REQUIREMENTS

### Must-Work Features (Verify Each)

| Feature | Shortcut | Handler Location | Status |
|---------|----------|-------------------|--------|
| Open File | Ctrl+O | line ~2845 | Verify |
| Save File | Ctrl+S | line ~2785 | Verify |
| Close Tab | Ctrl+W | line ~2805 | Verify |
| Undo | Ctrl+Z | line ~2500 | Verify |
| Redo | Ctrl+Shift+Z / Ctrl+Y | line ~2510 | Verify |
| Copy | Ctrl+C | line ~2700 | Verify |
| Cut | Ctrl+X | line ~2710 | Verify |
| Paste | Ctrl+V | line ~2720 | Verify |
| Select All | Ctrl+A | line ~2520 | Verify |
| Find | Ctrl+F | line ~2530 | Verify |
| Replace | Ctrl+H | line ~2540 | Verify |
| Command Palette | Ctrl+Shift+P / F1 | line ~2400 | Verify |
| Go to Line | Ctrl+G | line ~2547 | Verify |
| Toggle Terminal | Ctrl+` | line ~2570 | Verify |
| Toggle Sidebar | Ctrl+B | line ~2876 | Verify |
| New Tab | Ctrl+T | line ~2440 | Verify |
| Next/Prev Tab | Ctrl+Tab | line ~2420 | Verify |
| Toggle Comment | Ctrl+/ | line ~2550 | Verify |

**For each: run the app, test the shortcut, verify it works. Fix if broken.**

> **Note:** Line numbers are approximate. The file is 3100+ lines. Use `grep` to find exact locations.

---

## PHASE 6: INTERACTIVE DEBUGGING TOOLKIT

After all UI work is complete, create a **self-contained testing crate** that can be used to debug Forge.

### Crate: `forge-test-tools`

Create `forge/crates/forge-test-tools/` with:

```
forge-test-tools/
├── Cargo.toml
├── src/
│   ├── lib.rs          — public API
│   ├── screenshot.rs   — automated screenshot capture & diff
│   ├── input_sim.rs    — simulated mouse/keyboard events
│   ├── zone_debug.rs   — zone boundary visualization
│   └── assertions.rs   — visual assertions
```

#### 1. `screenshot.rs` — Automated Screenshot & Visual Diff

```rust
/// Capture a screenshot of the current frame
pub fn capture_frame(app: &mut Application) -> image::RgbaImage { ... }

/// Compare two screenshots pixel-by-pixel
pub fn diff_images(a: &RgbaImage, b: &RgbaImage) -> DiffResult { ... }

/// Assert screenshots match within tolerance
pub fn assert_visual_match(actual: &RgbaImage, expected_path: &str, tolerance: f32) { ... }
```

#### 2. `input_sim.rs` — Simulated Input Events

Build `winit::event::WindowEvent` instances for testing:

```rust
/// Simulate a mouse click at (x, y)
pub fn click(x: f32, y: f32) -> WindowEvent { ... }

/// Simulate typing a string
pub fn type_text(text: &str) -> Vec<WindowEvent> { ... }

/// Simulate key combo (e.g., Ctrl+S)
pub fn key_combo(modifiers: ModifiersState, key: Key) -> WindowEvent { ... }

/// Simulate mouse drag from (x1,y1) to (x2,y2)
pub fn drag(x1: f32, y1: f32, x2: f32, y2: f32) -> Vec<WindowEvent> { ... }
```

#### 3. `zone_debug.rs` — Visual Zone Debugging

Draw colored overlay borders on all layout zones so you can visually verify alignment:

```rust
/// Enable zone debug overlay — draws colored borders on every zone
pub fn enable_zone_overlay(state: &mut AppState) { ... }

/// Print all zone boundaries to console
pub fn dump_zones(layout: &LayoutZones) { ... }

/// Check if a point (x, y) falls in the expected zone
pub fn identify_zone(layout: &LayoutZones, x: f32, y: f32) -> &str { ... }
```

#### 4. `assertions.rs` — Behavioral Assertions

```rust
/// Assert that clicking at (x, y) opens the expected file
pub fn assert_click_opens_file(app: &mut Application, x: f32, y: f32, expected_file: &str) { ... }

/// Assert that typing text appears in the editor
pub fn assert_typing_works(app: &mut Application, text: &str) { ... }

/// Assert that a keyboard shortcut triggers the expected action
pub fn assert_shortcut(app: &mut Application, shortcut: KeyCombo, expected_state: AppStatePredicate) { ... }
```

### Integration: CLI Debug Mode

Add `--debug-zones` flag to the main application:

```rust
// In main.rs, add:
"--debug-zones" => { debug_zones = true; }

// In render(), when debug_zones is true:
if debug_zones {
    forge_test_tools::zone_debug::enable_zone_overlay(state);
}
```

This draws colored borders on every clickable zone so you can visually verify that the rendered UI and the click targets align.

### Integration: Test Harness

Add integration tests in `forge/tests/`:

```rust
#[test]
fn test_startup_screenshot() {
    let app = Application::new(None, Some("test_startup.png".into()));
    // Run one frame
    // Compare against golden screenshot
}

#[test]  
fn test_click_file_explorer() {
    let mut app = create_test_app();
    // Open sidebar
    app.handle_event(input_sim::key_combo(ctrl, Key::Character("b")));
    // Click first file
    let file_y = 44.0 + 22.0; // header + first item
    app.handle_event(input_sim::click(100.0, file_y));
    // Assert file opened
    assert!(app.state.tab_manager.tab_count() > 0);
}

#[test]
fn test_keyboard_input() {
    let mut app = create_test_app();
    for event in input_sim::type_text("hello world") {
        app.handle_event(event);
    }
    let text = app.state.tab_manager.active_editor().unwrap().text();
    assert!(text.contains("hello world"));
}
```

---

## VERIFICATION GATES

After **each phase**, you MUST:

1. `cargo check -p forge-app` — must compile clean
2. `cargo run -p forge-app -- --screenshot screenshot_phaseN.png` — take screenshot
3. Compare screenshot against VS Code reference
4. List any remaining visual differences
5. **Do NOT proceed to next phase until current phase passes**

### Final Verification Checklist

- [ ] App launches without errors
- [ ] Startup shows Welcome tab with content (not blank)
- [ ] Sidebar is closed by default
- [ ] Clicking Explorer icon opens sidebar and loads files
- [ ] Clicking a file opens it in a new tab
- [ ] Mouse click in editor positions cursor correctly
- [ ] Typing works immediately without clicking first
- [ ] All Ctrl+key shortcuts work
- [ ] Colors match VS Code Dark+ theme
- [ ] Status bar shows correct info
- [ ] Screenshot diff vs VS Code < 10% difference
- [ ] `forge-test-tools` crate compiles
- [ ] `--debug-zones` flag works
- [ ] At least 3 integration tests pass

---

## FILE REFERENCE — ALL 73 SOURCE FILES

### Core (Must Understand)
| File | Purpose | Lines |
|------|---------|-------|
| `application.rs` | Main app loop, rendering, input, init | ~3100 |
| `ui.rs` | Layout constants, zone computation | ~400 |
| `editor.rs` | Text buffer + cursor | ~388 |
| `tab_manager.rs` | Tab state management | ~180 |
| `main.rs` | CLI args + event loop | ~140 |

### UI Components (Must Verify)
| File | Purpose |
|------|---------|
| `activity_bar.rs` | Left icon bar |
| `tab_bar.rs` | Tab strip |
| `gutter.rs` | Line numbers |
| `status_bar.rs` | Bottom bar |
| `breadcrumb.rs` | Path breadcrumb |
| `scrollbar.rs` | Vertical scrollbar |
| `file_tree_ui.rs` | File tree hover/selection rects |
| `file_explorer.rs` | File system scanning |
| `cursor.rs` | Cursor rendering |
| `rect_renderer.rs` | GPU rect batch renderer |
| `title_bar.rs` | Window title bar |

### Features (Verify Work)
| File | Purpose |
|------|---------|
| `command_palette.rs` | Ctrl+Shift+P command palette |
| `find_bar.rs` | Ctrl+F search |
| `replace_bar.rs` | Ctrl+H replace |
| `go_to_line.rs` | Ctrl+G go to line |
| `context_menu.rs` | Right-click menu |
| `autocomplete.rs` | Code completion |
| `search_panel.rs` | Project-wide search |
| `bottom_panel.rs` | Terminal/output panel |

### Advanced (Verify Compile)
| File | Purpose |
|------|---------|
| `code_folding.rs` | Region folding |
| `bracket_match.rs` | Bracket highlighting |
| `indent_guides.rs` | Indent guide lines |
| `comment_toggle.rs` | Ctrl+/ toggle comment |
| `git_gutter.rs` | Git diff markers |
| `git_panel.rs` | Source control panel |
| `minimap.rs` | Code minimap |
| `debug_ui.rs` | Debug interface |
| `diff_view.rs` | Diff viewer |
| `multicursor.rs` | Multi-cursor editing |
| `word_wrap.rs` | Soft wrap |
| `emmet.rs` | Emmet abbreviations |
| `snippets.rs` | Code snippets |

### Crates (Dependencies)
| Crate | Purpose |
|-------|---------|
| `forge-core` | Buffer, Selection, Position, Transaction |
| `forge-syntax` | Tree-sitter highlighting |
| `forge-theme` | Color theme system |
| `forge-icons` | File/UI icons |
| `forge-lsp` | LSP client |
| `forge-terminal` | Terminal emulator |
| `forge-agent` | AI agent |
| `forge-debug` | DAP debugger |
| `forge-renderer` | wgpu rendering helpers |

---

## ANTI-CRASH RULES

1. **Never use `unwrap()` on user data** — always handle errors
2. **Never index arrays without bounds checking** — use `.get()`
3. **The `application.rs` file is 3100+ lines** — read it in sections, not all at once
4. **Borrow checker:** `state.font_system` is borrowed exclusively for text operations — never hold a ref while modifying other state fields
5. **wgpu API:** Surface texture must be presented exactly once per frame
6. **winit:** Event loop runs on main thread — no blocking I/O in event handlers
7. **glyphon:** Buffers must be shaped before rendering — call `shape_until_scroll()` after `set_text()`

---

## SUCCESS CRITERIA

The Forge IDE window must be **indistinguishable from VS Code** at first glance:
- Same dark background colors
- Same layout proportions
- Same menu bar labels
- Same status bar format
- Working file explorer with correct click targets
- Working keyboard shortcuts
- Working editor with cursor, selection, scrolling
- Interactive testing tools built and functional
