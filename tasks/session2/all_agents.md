# Session 2 — Editor Intelligence + Panels (All 10 Agents)

> **Read `tasks/GLOBAL_RULES.md` first. Each agent below runs in parallel. Zero file conflicts.**

---

## Agent 01 — Find Bar (Ctrl+F) + Replace Bar (Ctrl+H)

### `crates/forge-app/src/find_bar.rs`
Find overlay: text input capturing keys, Enter/Shift+Enter next/prev match, Escape to close. Regex toggle, case toggle, whole word toggle. `FindBar` struct with `open()`, `close()`, `search(query) -> Vec<Match>`, `next_match()`, `prev_match()`. Match struct: `{ line: usize, start_col: usize, end_col: usize }`. Highlight all matches via DecorationLayer.

### `crates/forge-app/src/replace_bar.rs`
Extends find: replace input, `replace_current()`, `replace_all()`. Undo support — all replaces wrapped in a single Transaction. Tests: search "hello" in "hello world hello" returns 2 matches; replace_all works.

---

## Agent 02 — Multi-cursor + Bracket Matching

### `crates/forge-app/src/multicursor.rs`
`MultiCursor` struct managing `Vec<Selection>`. Methods: `add_cursor(line, col)`, `select_next_occurrence(buffer, word)` (Ctrl+D), `select_all_occurrences(buffer, word)` (Ctrl+Shift+L). All cursors type simultaneously — produce multiple `Transaction` changes merged into one.

### `crates/forge-app/src/bracket_match.rs`
`BracketMatcher::find_match(buffer, pos) -> Option<Position>`. Match `()[]{}`. Skip brackets inside strings/comments (use basic state machine). Highlight matching bracket via decoration. Test: `fn foo(a: (b, c))` at col 6 matches closing `)` at end.

---

## Agent 03 — Indent Guides + Code Folding

### `crates/forge-app/src/indent_guides.rs`
`IndentGuides::compute(buffer, visible_range) -> Vec<GuideLine>`. GuideLine: `{ x: f32, y_start: f32, y_end: f32, active: bool }`. Detect indent level per line (count leading spaces / tab_size). Active guide = guide containing cursor. Render as 1px wide rectangles.

### `crates/forge-app/src/code_folding.rs`
`FoldingManager` tracking `Vec<FoldRange>`. `FoldRange { start_line, end_line, folded: bool }`. Compute fold ranges from indentation levels (simple heuristic: block starts when indent increases, ends when it returns). `toggle_fold(line)`, `fold_all()`, `unfold_all()`. Gutter indicators ▸/▾.

---

## Agent 04 — Minimap + Word Wrap

### `crates/forge-app/src/minimap.rs`
`Minimap` struct. `render(buffer, syntax_spans, scroll_top, total_lines, zone) -> Vec<Rect>`. Each line = 2px tall colored rectangle. Viewport indicator rectangle. Click on minimap = jump to that line. Hover = preview tooltip position.

### `crates/forge-app/src/word_wrap.rs`
`WordWrapper::wrap(line: &str, max_width_chars: usize) -> Vec<WrappedLine>`. `WrappedLine { text: String, original_line: usize, is_continuation: bool }`. Wrap at word boundaries when possible, hard-wrap otherwise. Toggle via `Alt+Z`. Gutter shows `↪` for continuation lines.

---

## Agent 05 — Bottom Panel + Problems Panel

### `crates/forge-app/src/bottom_panel.rs`
`BottomPanel { visible: bool, height: f32, active_tab: PanelTab, tabs: Vec<PanelTab> }`. `PanelTab` enum: Problems, Output, Terminal, DebugConsole. `toggle()` (Ctrl+`), `resize(delta)`, `set_tab(tab)`. Drag border to resize (min 100px, max 50% window).

### `crates/forge-app/src/problems_panel.rs`
`Diagnostic { file: String, line: usize, col: usize, message: String, severity: Severity }`. `Severity` enum: Error, Warning, Info, Hint. `ProblemsPanel { diagnostics: Vec<Diagnostic> }`. `add(d)`, `clear()`, `filter(severity)`, `count_by_severity() -> (usize, usize, usize)`. Click jumps to error line. Group by file.

---

## Agent 06 — Output Panel + Notifications

### `crates/forge-app/src/output_panel.rs`
`OutputChannel { name: String, lines: Vec<String> }`. `OutputPanel { channels: Vec<OutputChannel>, active: usize }`. `create_channel(name)`, `append(channel, text)`, `clear(channel)`. Channel selector. Scrollable output view.

### `crates/forge-app/src/notifications.rs`
`Notification { id: u64, message: String, level: Level, created_at: Instant, action: Option<String> }`. `Level`: Info, Warning, Error. `NotificationManager { notifications: Vec<Notification>, next_id: u64 }`. `show(msg, level) -> id`, `dismiss(id)`. Auto-dismiss: 5s info, 10s warning, sticky error. `tick()` removes expired notifications.

---

## Agent 07 — Command Palette + File Picker

### `crates/forge-app/src/command_palette.rs`
`Command { id: String, label: String, shortcut: Option<String>, category: Option<String> }`. `CommandPalette { visible: bool, query: String, commands: Vec<Command>, filtered: Vec<usize> }`. `open()`, `close()`, `type_char(c)`, `select() -> Command`. Fuzzy filter: score = sum of matched char positions weighted by consecutive bonus. Register 50+ default commands (file.save, edit.undo, etc.).

### `crates/forge-app/src/file_picker.rs`
`FilePicker { visible: bool, query: String, files: Vec<String>, filtered: Vec<(usize, f64)> }`. `open()`, `search(query)`. Fuzzy filename match. Recently opened files boost. `Ctrl+P` to open. Preview selection highlight. Navigate with arrows, Enter to open.

---

## Agent 08 — Status Bar Segments + Breadcrumb Dropdowns

### `crates/forge-app/src/status_segments.rs`
`StatusSegment { label: String, tooltip: String, on_click: Option<SegmentAction> }`. `SegmentAction` enum: SwitchLanguage, SwitchEncoding, SwitchLineEnding, ShowGitBranch, ShowErrors. Build segments: `[branch] [errors/warnings] [line:col] [encoding] [line-ending] [language] [mode]`. Click segment = popup action.

### `crates/forge-app/src/breadcrumb_dropdown.rs`
`BreadcrumbDropdown { visible: bool, items: Vec<String>, selected: usize, anchor: (f32, f32) }`. Show on click of breadcrumb segment. Arrow keys navigate, Enter selects, Escape closes. Items = sibling files/folders at clicked breadcrumb level.

---

## Agent 09 — Context Menus + Title Bar

### `crates/forge-app/src/context_menu.rs`
`MenuItem { label: String, shortcut: Option<String>, action: String, separator: bool }`. `ContextMenu { visible: bool, x: f32, y: f32, items: Vec<MenuItem> }`. `show(x, y, items)`, `hide()`, `handle_click(y) -> Option<String>`. Editor context: Cut/Copy/Paste/Select All + separator + Go to Definition/Peek/Rename. Tab context: Close/Close Others/Close All.

### `crates/forge-app/src/title_bar.rs`
`TitleBar { menus: Vec<Menu>, active_menu: Option<usize> }`. `Menu { label: &str, items: Vec<MenuItem> }`. Menus: File (New/Open/Save/Save As/Close), Edit (Undo/Redo/Cut/Copy/Paste/Find/Replace), View (Command Palette/Toggle Sidebar/Toggle Terminal/Minimap). Render custom title bar. Min/Max/Close buttons on right.

---

## Agent 10 — Go to Line + Comment Toggle

### `crates/forge-app/src/go_to_line.rs`
`GoToLine { visible: bool, input: String }`. `open()`, `type_char(c)`, `confirm() -> Option<usize>`, `cancel()`. Parse input as `line` or `line:col`. Preview target line while typing. Ctrl+G keybinding.

### `crates/forge-app/src/comment_toggle.rs`
`CommentToggler::toggle_line(line: &str, lang: Language) -> String`. Language-aware: `//` for Rust/JS/TS/Go/C, `#` for Python/Shell/TOML/YAML, `--` for SQL/Lua, `<!-- -->` for HTML. `toggle_block(text, lang) -> String`. Multi-line: if all lines commented, uncomment; else comment all. Ctrl+/ keybinding.

---

**Acceptance for ALL agents**: `cargo check --workspace && cargo test --workspace` passes with 0 errors.
