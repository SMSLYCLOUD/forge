# SESSION 3 — Terminal + Git + Search
# ONE JULES TASK — Copy this ENTIRE file as one Jules prompt.
# PREREQUISITE: Session 2 must be merged first.
# ═══════════════════════════════════════════════════════════════

You are working on **Forge**, a GPU-accelerated code editor written in Rust. You will complete ALL 10 tasks below in sequence. Do them ALL.

---

## RULES (MANDATORY)

1. **Rust 2021 edition.** No `.unwrap()` in production. `Result<T, E>` + `thiserror`. `.unwrap()` only in `#[cfg(test)]`.
2. **Every public function** MUST have ≥1 unit test.
3. `cargo fmt` + `cargo clippy -- -D warnings` = zero warnings.
4. **Do NOT modify `crates/forge-app/src/application.rs`.**
5. UTF-8, LF. `///` for public API docs.
6. Add deps to ROOT `[workspace.dependencies]`, reference as `{ workspace = true }`.
7. When creating a new crate, add it to `[workspace] members` in ROOT `Cargo.toml`.

---

## TASK 1: Create forge-terminal Crate — PTY + ANSI Parser

**Create new crate `crates/forge-terminal/`.**

**`Cargo.toml`** deps: `anyhow`, `thiserror`, `tracing`. On Windows, add `windows` crate for ConPTY (or use a conditional dep).

**`src/pty.rs`:**
- `Pty` struct: manages a child process with PTY
- `spawn(command: &str, cols: u16, rows: u16) -> Result<Self>` — spawn shell subprocess. On Windows, use piped stdin/stdout as fallback if ConPTY unavailable.
- `write(data: &[u8]) -> Result<()>`, `read() -> Result<Vec<u8>>`, `resize(cols, rows) -> Result<()>`, `is_alive() -> bool`, `kill() -> Result<()>`
- Tests (≥2): spawn returns Ok (can use `echo` or simple command), is_alive after spawn.

**`src/ansi.rs`:**
- `TermEvent` enum: `Print(char)`, `SetColor { fg: Option<[u8;3]>, bg: Option<[u8;3]> }`, `MoveCursor(u16, u16)`, `ClearLine`, `ClearScreen`, `Newline`, `Bell`
- `AnsiParser` struct with state machine
- `parse(input: &[u8]) -> Vec<TermEvent>` — parse VT100/ANSI: handle SGR (`\x1b[31m` etc.), cursor movement, clear commands
- Tests (≥3): parse plain text, parse SGR color, parse clear screen.

## TASK 2: forge-terminal — Grid Buffer + Shell Detection

**`src/grid.rs`:**
- `Cell` struct: `ch: char`, `fg: [u8; 3]`, `bg: [u8; 3]`, `bold: bool`, `italic: bool`, `underline: bool`
- `TerminalGrid` struct: `cells: Vec<Vec<Cell>>`, `cols: u16`, `rows: u16`, `cursor_row: u16`, `cursor_col: u16`, `scrollback: Vec<Vec<Cell>>`
- `write_char(c: char)`, `newline()`, `clear_line()`, `clear_screen()`, `resize(cols, rows)`, `scroll_up()`
- Scrollback: 10,000 lines max
- Tests (≥3): write_char places cell, newline moves cursor, resize works.

**`src/shell.rs`:**
- `detect_shell() -> String` — check `SHELL` env (Unix), find `pwsh.exe` / `powershell.exe` / `cmd.exe` (Windows)
- Tests (≥1): returns non-empty string.

**`src/lib.rs`:**
- `Terminal` struct composing PTY + Grid + Parser
- `Terminal::new() -> Result<Self>`, `send_input(text: &str) -> Result<()>`, `render_grid() -> &TerminalGrid`
- Re-export all public types.

## TASK 3: Terminal UI Rendering

**Create `crates/forge-app/src/terminal_ui.rs`:**
- `TerminalUi` struct for rendering `TerminalGrid` into the bottom panel
- `render(grid: &TerminalGrid, zone_x: f32, zone_y: f32, cell_width: f32, cell_height: f32) -> Vec<RenderCommand>` — produce render commands for each cell
- `RenderCommand` struct or enum: `DrawChar { x, y, ch, fg_color, bg_color }`, `DrawCursor { x, y }`
- `handle_key_input(key: char) -> Option<String>` — convert keypress to terminal input string
- Tests (≥2): render produces correct number of commands, key mapping works.

## TASK 4: Git Status Panel + Gutter Diff

**Create `crates/forge-app/src/git_panel.rs`:**
- `FileStatus` enum: `Modified`, `Added`, `Deleted`, `Untracked`, `Renamed`
- `GitFile` struct: `path: String`, `status: FileStatus`
- `GitPanel` struct: `files: Vec<GitFile>`, `repo_path: Option<String>`
- `refresh(repo_path: &str) -> Result<Vec<GitFile>>` — use `git2` crate to read repo status
- `stage_file(repo_path: &str, file: &str) -> Result<()>`
- `unstage_file(repo_path: &str, file: &str) -> Result<()>`
- Tests (≥2): GitFile creation, FileStatus display.

**Ensure `git2 = { workspace = true }` is in `crates/forge-app/Cargo.toml`.**

**Create `crates/forge-app/src/git_gutter.rs`:**
- `DiffKind` enum: `Added`, `Modified`, `Deleted`
- `GutterMark` struct: `line: usize`, `kind: DiffKind`
- `compute_gutter_marks(old_text: &str, new_text: &str) -> Vec<GutterMark>` — simple line-by-line diff
- Tests (≥2): added lines detected, deleted lines detected.

## TASK 5: Diff Viewer + Git Blame

**Create `crates/forge-app/src/diff_view.rs`:**
- `DiffLineKind` enum: `Added`, `Removed`, `Context`
- `DiffLine` struct: `kind: DiffLineKind`, `text: String`, `line_number: Option<usize>`
- `DiffHunk` struct: `old_start: usize`, `new_start: usize`, `lines: Vec<DiffLine>`
- `compute_diff(old: &str, new: &str) -> Vec<DiffHunk>` — simple LCS-based line diff
- Tests (≥2): identical texts = no hunks, added lines create hunks.

**Create `crates/forge-app/src/git_blame.rs`:**
- `BlameLine` struct: `commit_hash: String`, `author: String`, `date: String`, `line_number: usize`
- `blame_file(repo_path: &str, file_path: &str) -> Result<Vec<BlameLine>>` — use `git2::Repository::blame_file()`
- Tests (≥1): BlameLine struct creation.

## TASK 6: Git Branch Manager

**Create `crates/forge-app/src/git_branch.rs`:**
- `BranchInfo` struct: `name: String`, `is_current: bool`, `is_remote: bool`
- `BranchManager` struct
- `list_branches(repo_path: &str) -> Result<Vec<BranchInfo>>` — use `git2`
- `current_branch(repo_path: &str) -> Result<Option<String>>`
- `create_branch(repo_path: &str, name: &str) -> Result<()>`
- Tests (≥2): BranchInfo creation, current_branch format.

## TASK 7: Create forge-search Crate — Fuzzy Finder + Content Search

**Create new crate `crates/forge-search/`.**

**`Cargo.toml`** deps: `anyhow`, `serde`, `thiserror`. Add `ignore = "0.4"` to ROOT workspace deps.

**`src/fuzzy.rs`:**
- `fuzzy_score(query: &str, candidate: &str) -> Option<f64>` — scan candidate for query chars. Bonuses: consecutive (+10), word boundary (+8), filename (+5). Penalty: gaps (-1). Return None if not all chars found.
- `fuzzy_filter(query: &str, candidates: &[&str]) -> Vec<(usize, f64)>` — sorted desc by score
- Tests (≥3): exact match highest score, partial match scores, no match returns None.

**`src/content.rs`:**
- `SearchOpts` struct: `regex: bool`, `case_sensitive: bool`, `whole_word: bool`
- `SearchResult` struct: `file: String`, `line: usize`, `col: usize`, `text: String`
- `search_in_text(text: &str, query: &str, opts: &SearchOpts) -> Vec<SearchResult>` — search within a single text
- `search_directory(root: &Path, query: &str, opts: &SearchOpts) -> Result<Vec<SearchResult>>` — walk files, skip binary, respect .gitignore
- Tests (≥3): find in text, case insensitive, whole word.

**`src/lib.rs`:** re-export.

Add `"crates/forge-search"` to workspace members.

## TASK 8: Search Results Panel

**Create `crates/forge-app/src/search_panel.rs`:**
- `SearchPanel` struct: `visible: bool`, `query: String`, `results: Vec<SearchResult>`, `selected: usize`, `regex: bool`, `case_sensitive: bool`, `whole_word: bool`
- `search(root: &Path, query: &str)` — calls forge-search
- `next_result()`, `prev_result()`, `selected_result() -> Option<&SearchResult>`
- `toggle_regex()`, `toggle_case()`, `toggle_whole_word()`
- Tests (≥2): SearchPanel creation, toggle methods.

**Add `forge-search = { workspace = true }` to `crates/forge-app/Cargo.toml`** (add `forge-search` to workspace deps in root first).

## TASK 9: Go to Definition + References

**Create `crates/forge-app/src/go_to_def.rs`:**
- `Location` struct: `file: String`, `line: usize`, `col: usize`
- `NavStack` struct: `history: Vec<Location>`, `position: usize`
- `push(loc)`, `back() -> Option<&Location>`, `forward() -> Option<&Location>`
- `find_definition(symbol: &str, workspace_files: &[(String, String)]) -> Option<Location>` — text search for `fn {symbol}`, `struct {symbol}`, `class {symbol}`, `def {symbol}`
- Tests (≥3): find fn definition, NavStack back/forward, not found returns None.

**Create `crates/forge-app/src/references.rs`:**
- `find_references(symbol: &str, text: &str) -> Vec<Location>` — find all occurrences of symbol as whole word
- Tests (≥2): finds multiple references, excludes partial matches.

## TASK 10: Symbol Outline + Workspace Symbols

**Create `crates/forge-app/src/outline_panel.rs`:**
- `SymbolKind` enum: `Function`, `Struct`, `Enum`, `Impl`, `Class`, `Method`, `Variable`
- `Symbol` struct: `name: String`, `kind: SymbolKind`, `line: usize`, `children: Vec<Symbol>`
- `extract_symbols(text: &str) -> Vec<Symbol>` — regex-based: find `fn name`, `struct Name`, `enum Name`, `impl Name`, `class Name`, `def name`
- Tests (≥2): extract Rust symbols, extract Python symbols.

**Create `crates/forge-app/src/workspace_symbols.rs`:**
- `WorkspaceSymbol` struct: `name: String`, `kind: SymbolKind`, `file: String`, `line: usize`
- `WorkspaceSymbolIndex` struct: `symbols: Vec<WorkspaceSymbol>`
- `build_index(files: &[(String, String)]) -> Self` — scan all files for symbols
- `search(query: &str) -> Vec<&WorkspaceSymbol>` — fuzzy match symbol names
- Tests (≥2): build index from file content, search finds matches.

---

## FINAL VERIFICATION

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test --workspace
cargo check --workspace
```

ALL FOUR must exit 0. **Do NOT modify `application.rs`.**

Add `pub mod` declarations for ALL new files to their respective crate's entry file.
