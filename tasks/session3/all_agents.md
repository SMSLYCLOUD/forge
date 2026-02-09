# Session 3 — Terminal + Git + Search (All 10 Agents)

> **Read `tasks/GLOBAL_RULES.md` first. Each agent runs in parallel. Zero file conflicts.**

---

## Agent 01 — forge-terminal: PTY + ANSI Parser

### New crate `crates/forge-terminal/`
Add dependencies: `anyhow`, `thiserror`, `tracing`. On Windows, use `windows` crate for ConPTY.

### `crates/forge-terminal/src/pty.rs`
`Pty` struct: `spawn(command: &str, cols: u16, rows: u16) -> Result<Self>`. Methods: `write(data: &[u8])`, `read() -> Vec<u8>`, `resize(cols, rows)`, `is_alive() -> bool`, `kill()`. On Windows: use `CreatePseudoConsole` via `windows` crate or shell out to `conpty.dll`. Fallback: spawn process with piped stdin/stdout.

### `crates/forge-terminal/src/ansi.rs`
`AnsiParser` state machine. `parse(input: &[u8]) -> Vec<TermEvent>`. `TermEvent` enum: `Print(char)`, `SetColor(fg, bg)`, `MoveCursor(row, col)`, `ClearLine`, `ClearScreen`, `ScrollUp`, `ScrollDown`, `Bell`. Handle SGR sequences (`\x1b[31m` etc.) for 16-color, 256-color, and truecolor.

---

## Agent 02 — forge-terminal: Grid Buffer + Shell Detection

### `crates/forge-terminal/src/grid.rs`
`Cell { ch: char, fg: [u8; 3], bg: [u8; 3], bold: bool, italic: bool, underline: bool }`. `TerminalGrid { cells: Vec<Vec<Cell>>, cols: u16, rows: u16, cursor_row: u16, cursor_col: u16, scrollback: Vec<Vec<Cell>> }`. Methods: `write_char(c)`, `newline()`, `clear_line()`, `clear_screen()`, `resize(cols, rows)`, `scroll_up()`. Scrollback buffer: 10,000 lines.

### `crates/forge-terminal/src/shell.rs`
`detect_shell() -> String`: Check `SHELL` env (Unix), find `pwsh.exe` then `powershell.exe` then `cmd.exe` (Windows).

### `crates/forge-terminal/src/lib.rs`
Compose into `Terminal` struct: PTY + Grid + Parser. `Terminal::new() -> Result<Self>`, `send_input(text)`, `tick() -> Vec<TermEvent>`, `render_grid() -> &TerminalGrid`.

---

## Agent 03 — Terminal UI Rendering

### `crates/forge-app/src/terminal_ui.rs`
`TerminalUi` struct rendering `TerminalGrid` into the bottom panel zone. Character-by-character glyphon text rendering with per-cell colors. Block cursor rendering. Mouse wheel scrollback. Selection for copy. Input routing: when terminal focused, keys go to PTY; Escape returns focus to editor. Add `mod terminal_ui;` to main.rs.

---

## Agent 04 — Git Status Panel + Gutter Diff

### `crates/forge-app/src/git_panel.rs`
`GitPanel` struct using `git2`. `refresh(repo_path) -> Result<Vec<GitFile>>`. `GitFile { path: String, status: FileStatus }`. `FileStatus`: Modified, Added, Deleted, Untracked, Renamed. Group by staged/unstaged. Stage button: `git2::Index::add_path()`. Unstage: `git2::Index::remove_path()`. Commit: `repo.commit()`. Render as list with status icons M/A/D/U.

### `crates/forge-app/src/git_gutter.rs`
`GutterDiff::compute(repo_path, file_path) -> Result<Vec<GutterMark>>`. `GutterMark { line: usize, kind: DiffKind }`. `DiffKind`: Added (green), Modified (blue), Deleted (red triangle). Use `git2::Repository::diff_index_to_workdir()`. Render marks as colored bars in gutter (3px wide, left edge).

---

## Agent 05 — Diff Viewer + Git Blame

### `crates/forge-app/src/diff_view.rs`
`DiffView` struct: side-by-side view of old vs new file. `compute_diff(old: &str, new: &str) -> Vec<DiffHunk>`. `DiffHunk { old_start, old_count, new_start, new_count, lines: Vec<DiffLine> }`. `DiffLine { kind: Added/Removed/Context, text: String }`. Synchronized scrolling. Navigate hunks: next/prev.

### `crates/forge-app/src/git_blame.rs`
`BlameView::blame(repo_path, file_path) -> Result<Vec<BlameLine>>`. `BlameLine { commit_hash: String, author: String, date: String, line_number: usize }`. Use `git2::Repository::blame_file()`. Render as inline annotations (dimmed text, right of line numbers). Toggle with Ctrl+Shift+G.

---

## Agent 06 — Git Branch Manager

### `crates/forge-app/src/git_branch.rs`
`BranchManager` struct. `list_branches(repo) -> Vec<BranchInfo>`. `BranchInfo { name, is_current, is_remote, ahead, behind }`. `create_branch(name)`, `checkout(name)`, `delete(name)`. Status bar dropdown showing current branch. Use `git2::Repository::branches()`, `git2::Repository::set_head()`.

---

## Agent 07 — forge-search: Fuzzy Finder + Content Search

### New crate `crates/forge-search/`
Dependencies: `anyhow`, `serde`.

### `crates/forge-search/src/fuzzy.rs`
`fuzzy_score(query: &str, candidate: &str) -> Option<f64>`. Algorithm: scan candidate for query chars in order. Score bonuses: consecutive match (+10), word boundary match (+8), filename match (+5). Penalty for gaps (-1 per gap). Return `None` if not all chars found. `fuzzy_filter(query, candidates) -> Vec<(usize, f64)>` sorted by score desc. Benchmark target: <5ms for 100k items.

### `crates/forge-search/src/content.rs`
`ContentSearcher::search(root: &Path, query: &str, opts: SearchOpts) -> Vec<SearchResult>`. `SearchOpts { regex: bool, case_sensitive: bool, whole_word: bool, include_glob: Option<String>, exclude_glob: Option<String> }`. `SearchResult { file: String, line: usize, col: usize, text: String }`. Recursive file walk, skip binary files, respect .gitignore via `ignore` crate. Add `ignore = "0.4"` to workspace deps.

---

## Agent 08 — Search Results Panel

### `crates/forge-app/src/search_panel.rs`
`SearchPanel { visible: bool, query: String, results: Vec<SearchResult>, selected: usize }`. Sidebar view: text input with regex/case/word toggles. Live results (debounced 300ms). Results grouped by file with match count. Click = jump to file:line. Replace input + Replace All button. `search()` calls `forge-search::ContentSearcher`.

---

## Agent 09 — Go to Definition + References

### `crates/forge-app/src/go_to_def.rs`
`GoToDefinition::find(buffer, position, workspace_files) -> Option<Location>`. Tree-sitter: extract symbol at cursor. Text search for `fn {symbol}`, `struct {symbol}`, `class {symbol}`, `def {symbol}` across workspace files. Navigation stack: `NavStack { history: Vec<Location>, position: usize }`. `push()`, `back()`, `forward()`. F12 keybinding. Ctrl+Click.

### `crates/forge-app/src/references.rs`
`ReferenceFinder::find(symbol: &str, workspace_root: &Path) -> Vec<Location>`. Text-based search for all occurrences across workspace. Filter: exclude declarations (heuristic: if preceded by `fn`/`struct`/etc). Shift+F12 keybinding. Results in peek view or search panel.

---

## Agent 10 — Symbol Outline + Workspace Symbols

### `crates/forge-app/src/outline_panel.rs`
`OutlinePanel::extract_symbols(tree, source) -> Vec<Symbol>`. `Symbol { name: String, kind: SymbolKind, line: usize, children: Vec<Symbol> }`. `SymbolKind`: Function, Struct, Enum, Impl, Class, Method, Variable. Use tree-sitter: walk nodes, extract named function_item, struct_item, enum_item, impl_item. Sidebar panel. Click = jump.

### `crates/forge-app/src/workspace_symbols.rs`
`WorkspaceSymbolIndex { symbols: Vec<(String, SymbolKind, String, usize)> }`. Build by scanning all files with tree-sitter. `search(query) -> Vec<WorkspaceSymbol>`. Ctrl+T keybinding. Fuzzy match symbol names. Cached, rebuild on file save.

---

**Acceptance**: `cargo check --workspace && cargo test --workspace` passes.
