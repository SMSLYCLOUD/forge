# SESSION 2 — Editor Intelligence + Panels UI
# ONE JULES TASK — Copy this ENTIRE file as one Jules prompt.
# PREREQUISITE: Session 1 must be merged first.
# ═══════════════════════════════════════════════════════════════

You are working on **Forge**, a GPU-accelerated code editor written in Rust. You will complete ALL 10 tasks below in sequence. Each task creates NEW files in `crates/forge-app/src/`. Do them ALL.

---

## RULES (MANDATORY)

1. **Rust 2021 edition.** No `.unwrap()` in production. `Result<T, E>` + `thiserror`. `.unwrap()` only in `#[cfg(test)]`.
2. **Every public function** MUST have ≥1 unit test.
3. `cargo fmt` + `cargo clippy -- -D warnings` after completing ALL tasks. Zero warnings.
4. **Do NOT modify `crates/forge-app/src/application.rs`.** That file is only modified in Session 5.
5. UTF-8, LF line endings. `///` for public API docs.
6. Add new deps to ROOT `[workspace.dependencies]`, reference as `{ workspace = true }`.
7. For each new source file, add `pub mod filename;` to the crate's `lib.rs` or `main.rs`.

---

## WORKSPACE CONTEXT

After Session 1, the workspace has these crates: `forge-core`, `forge-renderer`, `forge-window`, `forge-app`, `forge-config`, `forge-theme`, `forge-input`, `forge-surfaces`, `forge-confidence`, `forge-propagation`, `forge-semantic`, `forge-bayesnet`, `forge-ml`, `forge-anticipation`, `forge-immune`, `forge-developer`, `forge-feedback`, `forge-agent`, `forge-net`, `forge-types`, `forge-keybindings`, `forge-workspace`.

All files in this session go in `crates/forge-app/src/` unless stated otherwise.

---

## TASK 1: Find Bar (Ctrl+F)

**Create `crates/forge-app/src/find_bar.rs`:**

- `Match` struct: `line: usize`, `start_col: usize`, `end_col: usize`
- `FindBar` struct: `visible: bool`, `query: String`, `matches: Vec<Match>`, `current_match: usize`, `case_sensitive: bool`, `regex_mode: bool`, `whole_word: bool`
- `open()`, `close()`, `search(text: &str, query: &str) -> Vec<Match>`, `next_match() -> Option<&Match>`, `prev_match() -> Option<&Match>`, `set_case_sensitive(bool)`, `set_regex(bool)`, `set_whole_word(bool)`, `match_count() -> usize`
- Tests (≥3): search returns correct matches, next/prev cycle, case sensitivity toggle.

## TASK 2: Replace Bar (Ctrl+H)

**Create `crates/forge-app/src/replace_bar.rs`:**

- `ReplaceBar` struct: extends find bar concept with `replace_text: String`
- `replace_current(text: &mut String, find: &str, replace: &str, match_pos: &Match) -> String` — replace single match
- `replace_all(text: &str, find: &str, replace: &str) -> (String, usize)` — returns new text + count
- Tests (≥2): replace_current works, replace_all returns correct count.

## TASK 3: Multi-cursor

**Create `crates/forge-app/src/multicursor.rs`:**

- `Selection` struct: `start: Position`, `end: Position` (use or define Position with line/col)
- `MultiCursor` struct: `cursors: Vec<Selection>`
- `add_cursor(line, col)`, `select_next_occurrence(text: &str, word: &str)` (Ctrl+D logic), `select_all_occurrences(text: &str, word: &str)` (Ctrl+Shift+L), `cursor_count() -> usize`, `clear()`
- Tests (≥3): add cursor, select_next finds occurrence, select_all finds all.

## TASK 4: Bracket Matching

**Create `crates/forge-app/src/bracket_match.rs`:**

- `BracketMatcher::find_match(text: &str, line: usize, col: usize) -> Option<(usize, usize)>` — finds matching bracket for `()[]{}`. Skip brackets inside strings/comments (use basic state machine: track `"` and `//`/`/* */`).
- Tests (≥3): match `(` to `)`, nested brackets, brackets inside strings are skipped.

## TASK 5: Indent Guides

**Create `crates/forge-app/src/indent_guides.rs`:**

- `GuideLine` struct: `col: usize`, `start_line: usize`, `end_line: usize`, `active: bool`
- `IndentGuides::compute(text: &str, tab_size: u32, cursor_line: usize) -> Vec<GuideLine>` — detect indent levels per line, return vertical guide positions. The guide containing the cursor line is `active = true`.
- Tests (≥2): simple indented block returns guides, active guide is correct.

## TASK 6: Code Folding

**Create `crates/forge-app/src/code_folding.rs`:**

- `FoldRange` struct: `start_line: usize`, `end_line: usize`, `folded: bool`
- `FoldingManager` struct: `ranges: Vec<FoldRange>`
- `compute_ranges(text: &str) -> Vec<FoldRange>` — indentation-based: block starts when indent increases, ends when it returns to original level.
- `toggle_fold(line: usize)`, `fold_all()`, `unfold_all()`, `is_line_visible(line: usize) -> bool`
- Tests (≥3): compute_ranges finds blocks, toggle fold hides lines, unfold_all resets.

## TASK 7: Minimap

**Create `crates/forge-app/src/minimap.rs`:**

- `MinimapLine` struct: `line: usize`, `color: [f32; 3]`
- `Minimap` struct: `lines: Vec<MinimapLine>`, `viewport_start: usize`, `viewport_end: usize`, `total_lines: usize`
- `build(total_lines: usize, viewport_start: usize, viewport_end: usize) -> Self`
- `click_to_line(y_fraction: f32) -> usize` — convert click position to line number
- Tests (≥2): click_to_line maps correctly, build has correct line count.

## TASK 8: Word Wrap

**Create `crates/forge-app/src/word_wrap.rs`:**

- `WrappedLine` struct: `text: String`, `original_line: usize`, `is_continuation: bool`
- `WordWrapper::wrap(line: &str, max_chars: usize) -> Vec<WrappedLine>` — wrap at word boundaries when possible, hard-wrap otherwise.
- `wrap_all(lines: &[&str], max_chars: usize) -> Vec<WrappedLine>`
- Tests (≥3): short line not wrapped, long line wrapped at word boundary, very long word hard-wrapped.

## TASK 9: Bottom Panel + Problems Panel

**Create `crates/forge-app/src/bottom_panel.rs`:**

- `PanelTab` enum: `Problems`, `Output`, `Terminal`, `DebugConsole`
- `BottomPanel` struct: `visible: bool`, `height: f32`, `active_tab: PanelTab`
- `toggle()`, `set_tab(tab)`, `resize(new_height: f32)` (clamp between 100.0 and max)

**Create `crates/forge-app/src/problems_panel.rs`:**

- `Severity` enum: `Error`, `Warning`, `Info`, `Hint`
- `Diagnostic` struct: `file: String`, `line: usize`, `col: usize`, `message: String`, `severity: Severity`
- `ProblemsPanel` struct: `diagnostics: Vec<Diagnostic>`
- `add(d)`, `clear()`, `filter(severity) -> Vec<&Diagnostic>`, `count_by_severity() -> (usize, usize, usize, usize)`
- Tests (≥3): add + count, filter by severity, clear empties list.

## TASK 10: Output Panel + Notifications + Command Palette + File Picker + Context Menu + Go to Line + Comment Toggle + Status Segments + Breadcrumbs + Title Bar

**Create these files** in `crates/forge-app/src/`:

### `output_panel.rs`
- `OutputChannel` struct: `name: String`, `lines: Vec<String>`
- `OutputPanel` struct: `channels: Vec<OutputChannel>`, `active: usize`
- `create_channel(name)`, `append(channel_idx, text)`, `clear(channel_idx)`

### `notifications.rs`
- `Level` enum: `Info`, `Warning`, `Error`
- `Notification` struct: `id: u64`, `message: String`, `level: Level`, `created_at: std::time::Instant`
- `NotificationManager` struct: `notifications: Vec<Notification>`, `next_id: u64`
- `show(msg, level) -> u64`, `dismiss(id)`, `tick()` — remove expired (5s info, 10s warning, never error)

### `command_palette.rs`
- `Command` struct: `id: String`, `label: String`, `shortcut: Option<String>`, `category: Option<String>`
- `CommandPalette` struct: `visible: bool`, `query: String`, `commands: Vec<Command>`, `filtered: Vec<usize>`
- `open()`, `close()`, `type_char(c: char)`, `backspace()`, `select(idx) -> Option<&Command>`
- Fuzzy filter: score = sum of matched char positions weighted by consecutive bonus
- Register 30+ default commands: file.save, file.open, edit.undo, edit.redo, etc.

### `file_picker.rs`
- `FilePicker` struct: `visible: bool`, `query: String`, `files: Vec<String>`, `filtered: Vec<(usize, f64)>`
- `open()`, `close()`, `search(query)`, `select(idx) -> Option<&str>`
- Fuzzy filename matching. Recently opened files boost.

### `context_menu.rs`
- `MenuItem` struct: `label: String`, `shortcut: Option<String>`, `action: String`, `separator: bool`
- `ContextMenu` struct: `visible: bool`, `x: f32`, `y: f32`, `items: Vec<MenuItem>`
- `show(x, y, items)`, `hide()`, `handle_click(idx) -> Option<String>`
- Editor context: Cut/Copy/Paste/Select All + separator + Go to Definition/Rename

### `go_to_line.rs`
- `GoToLine` struct: `visible: bool`, `input: String`
- `open()`, `type_char(c)`, `confirm() -> Option<(usize, Option<usize>)>` — parse "line" or "line:col"
- `cancel()`

### `comment_toggle.rs`
- `Language` enum: `Rust`, `JavaScript`, `Python`, `Html`, `Css`, `Go`, `Other`
- `CommentToggler::toggle_line(line: &str, lang: Language) -> String` — add/remove `//`, `#`, `<!-- -->` etc.
- `toggle_block(lines: &[&str], lang: Language) -> Vec<String>` — if all commented, uncomment; else comment all.

### `status_segments.rs`
- `StatusSegment` struct: `label: String`, `tooltip: String`
- `build_segments(branch: &str, errors: usize, warnings: usize, line: usize, col: usize, encoding: &str, line_ending: &str, language: &str) -> Vec<StatusSegment>`

### `breadcrumb_dropdown.rs`
- `BreadcrumbDropdown` struct: `visible: bool`, `items: Vec<String>`, `selected: usize`
- `show(items)`, `hide()`, `select(idx) -> Option<&str>`, `move_up()`, `move_down()`

### `title_bar.rs`
- `Menu` struct: `label: String`, `items: Vec<MenuItem>` (reuse MenuItem from context_menu or define locally)
- `TitleBar` struct: `menus: Vec<Menu>`, `active_menu: Option<usize>`
- `open_menu(idx)`, `close_menu()`, `handle_click(idx) -> Option<String>`
- Default menus: File (New/Open/Save/Close), Edit (Undo/Redo/Cut/Copy/Paste), View (Command Palette/Toggle Sidebar/Toggle Terminal)

**Add `pub mod` declarations for ALL new files** to `crates/forge-app/src/main.rs` or `lib.rs`.

**Each file must have ≥2 tests.**

---

## FINAL VERIFICATION

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test --workspace
cargo check --workspace
```

ALL FOUR must exit 0. **Do NOT modify `application.rs`.**
