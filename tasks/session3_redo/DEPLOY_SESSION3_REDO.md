# SESSION 3 REDO — Terminal + Git + Search (Missing Deliverables)
# ONE JULES TASK — Copy this ENTIRE file as one Jules prompt.
# PREREQUISITE: The workspace already compiles. `cargo check --workspace` exits 0.
# THIS SESSION CREATES 2 NEW CRATES AND 10 NEW APP MODULES.
# ═══════════════════════════════════════════════════════════════

You are working on **Forge**, a GPU-accelerated code editor written in Rust. The workspace is at the repo root with `Cargo.toml` containing `[workspace]` with `members = [...]` and `[workspace.dependencies]`.

**CRITICAL CONTEXT:**
- Rust 2021 edition.
- ROOT `Cargo.toml` already has these workspace deps: `anyhow`, `thiserror`, `tracing`, `serde`, `serde_json`, `git2 = "0.20"`, `regex = "1"`, `tokio`, `futures`, `async-trait`.
- `crates/forge-app/Cargo.toml` already has: `forge-core`, `forge-renderer`, `forge-config`, `forge-theme`, `forge-syntax`, `forge-debug`, `forge-lsp`, `forge-plugin`, `ropey`, `wgpu`, `winit`, `glyphon`, `anyhow`, `tracing`, `tracing-subscriber`, `serde`, `toml`, `dirs-next`, `regex`.
- `crates/forge-app/src/main.rs` has `mod` declarations for all existing modules. You will ADD new `mod` lines for new modules.

**IMPORTANT RULES:**
1. No `.unwrap()` in production code. Use `Result<T, E>` + `thiserror` or `anyhow`. `.unwrap()` only in `#[cfg(test)]`.
2. Every public function MUST have ≥1 unit test.
3. `cargo fmt` + `cargo clippy -- -D warnings` = zero warnings.
4. **Do NOT modify `crates/forge-app/src/application.rs`.** Only create new files and modify `main.rs` to add `mod` declarations.
5. UTF-8, LF line endings. `///` for public API docs.
6. Add NEW deps to ROOT `Cargo.toml` under `[workspace.dependencies]`, reference in crate `Cargo.toml` as `{ workspace = true }`.
7. When creating a new crate, add it to `[workspace] members` in ROOT `Cargo.toml`.
8. All `pub mod` declarations for new modules go in `crates/forge-app/src/main.rs`.

---

## TASK 1: Create `forge-terminal` Crate — PTY + ANSI Parser

**Create new directory `crates/forge-terminal/`.**

**`crates/forge-terminal/Cargo.toml`:**
```toml
[package]
name = "forge-terminal"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true

[dependencies]
anyhow = { workspace = true }
thiserror = { workspace = true }
tracing = { workspace = true }
```

**Add `"crates/forge-terminal"` to `[workspace] members` in ROOT `Cargo.toml`.**

**`crates/forge-terminal/src/pty.rs`:**
- `Pty` struct: wraps `std::process::Child` with piped stdin/stdout
- `spawn(command: &str, _cols: u16, _rows: u16) -> Result<Self>` — spawn shell subprocess with piped stdin/stdout. Use `std::process::Command` with `.stdin(Stdio::piped())` and `.stdout(Stdio::piped())`.
- `write(&mut self, data: &[u8]) -> Result<()>` — write to child stdin
- `read(&mut self) -> Result<Vec<u8>>` — non-blocking read from child stdout (use `try_read` or set non-blocking if available, else return empty vec)
- `resize(&self, _cols: u16, _rows: u16) -> Result<()>` — no-op for now (TODO: ConPTY on Windows)
- `is_alive(&mut self) -> bool` — check `try_wait()`
- `kill(&mut self) -> Result<()>` — kill child process
- Tests (≥2): `Pty` struct creation compiles, `is_alive` logic verified with a mock

**`crates/forge-terminal/src/ansi.rs`:**
- `TermEvent` enum: `Print(char)`, `SetColor { fg: Option<[u8;3]>, bg: Option<[u8;3]> }`, `MoveCursor(u16, u16)`, `ClearLine`, `ClearScreen`, `Newline`, `Bell`
- `AnsiParser` struct with internal state: `state: ParserState` enum (`Ground`, `Escape`, `Csi`), `params: Vec<u16>`, `current_param: Option<u16>`
- `parse(&mut self, input: &[u8]) -> Vec<TermEvent>` — state machine:
  - `Ground`: regular chars → `Print(ch)`, `\n` → `Newline`, `\x07` → `Bell`, `\x1b` → transition to `Escape`
  - `Escape`: `[` → transition to `Csi`, anything else → back to `Ground`
  - `Csi`: digits build params, `;` separates params, `m` → SGR (SetColor), `H` → MoveCursor, `J` → ClearScreen (`params[0]==2`), `K` → ClearLine, any other letter → back to Ground
- SGR parsing: `30-37` → fg standard colors, `40-47` → bg standard colors, `38;2;r;g;b` → fg 24-bit, `48;2;r;g;b` → bg 24-bit, `0` → reset
- Standard color map: `[0,0,0], [205,49,49], [13,188,121], [229,229,16], [36,114,200], [188,63,188], [17,168,205], [229,229,229]`
- Tests (≥3): parse plain text returns Print events, parse `\x1b[31m` sets fg red, parse `\x1b[2J` returns ClearScreen

**`crates/forge-terminal/src/grid.rs`:**
- `Cell` struct with `Default`: `ch: char` (default `' '`), `fg: [u8; 3]` (default `[229,229,229]`), `bg: [u8; 3]` (default `[0,0,0]`), `bold: bool`, `italic: bool`, `underline: bool`
- `TerminalGrid` struct: `cells: Vec<Vec<Cell>>`, `cols: u16`, `rows: u16`, `cursor_row: u16`, `cursor_col: u16`, `scrollback: Vec<Vec<Cell>>`
- `TerminalGrid::new(cols: u16, rows: u16) -> Self` — init cells grid
- `write_char(&mut self, c: char)` — place char at cursor, advance cursor, wrap if at line end
- `newline(&mut self)` — move cursor to start of next line, scroll if needed
- `clear_line(&mut self)` — fill current row with default cells
- `clear_screen(&mut self)` — fill all rows with default cells
- `resize(&mut self, cols: u16, rows: u16)` — rebuild cells grid
- `scroll_up(&mut self)` — move top line to scrollback (max 10000), shift up, clear bottom
- `set_cursor_color(&mut self, fg: [u8; 3], bg: [u8; 3])` — set color for next writes
- Tests (≥3): write_char places cell, newline moves cursor, clear_screen empties grid

**`crates/forge-terminal/src/shell.rs`:**
- `detect_shell() -> String` — check `SHELL` env (Unix), then try `pwsh.exe`, `powershell.exe`, `cmd.exe` (Windows via `which` or hardcoded paths)
- Tests (≥1): returns non-empty string

**`crates/forge-terminal/src/lib.rs`:**
```rust
pub mod pty;
pub mod ansi;
pub mod grid;
pub mod shell;

pub use pty::Pty;
pub use ansi::{AnsiParser, TermEvent};
pub use grid::{Cell, TerminalGrid};
pub use shell::detect_shell;

/// High-level terminal combining PTY + Grid + Parser
pub struct Terminal {
    pty: Pty,
    parser: AnsiParser,
    pub grid: TerminalGrid,
}

impl Terminal {
    pub fn new() -> anyhow::Result<Self> {
        let shell = detect_shell();
        let pty = Pty::spawn(&shell, 80, 24)?;
        Ok(Self {
            pty,
            parser: AnsiParser::new(),
            grid: TerminalGrid::new(80, 24),
        })
    }

    pub fn send_input(&mut self, text: &str) -> anyhow::Result<()> {
        self.pty.write(text.as_bytes())
    }

    pub fn poll_output(&mut self) -> anyhow::Result<()> {
        let data = self.pty.read()?;
        if !data.is_empty() {
            let events = self.parser.parse(&data);
            for event in events {
                match event {
                    TermEvent::Print(c) => self.grid.write_char(c),
                    TermEvent::Newline => self.grid.newline(),
                    TermEvent::ClearLine => self.grid.clear_line(),
                    TermEvent::ClearScreen => self.grid.clear_screen(),
                    TermEvent::SetColor { fg, bg } => {
                        let fg_color = fg.unwrap_or([229, 229, 229]);
                        let bg_color = bg.unwrap_or([0, 0, 0]);
                        self.grid.set_cursor_color(fg_color, bg_color);
                    }
                    TermEvent::MoveCursor(row, col) => {
                        self.grid.cursor_row = row.min(self.grid.rows.saturating_sub(1));
                        self.grid.cursor_col = col.min(self.grid.cols.saturating_sub(1));
                    }
                    TermEvent::Bell => {} // ignore
                }
            }
        }
        Ok(())
    }

    pub fn render_grid(&self) -> &TerminalGrid {
        &self.grid
    }

    pub fn is_alive(&mut self) -> bool {
        self.pty.is_alive()
    }
}
```

Tests (≥2): Terminal struct compiles, grid dimensions correct after new().

---

## TASK 2: Create `forge-search` Crate — Fuzzy Finder + Content Search

**Create new directory `crates/forge-search/`.**

**`crates/forge-search/Cargo.toml`:**
```toml
[package]
name = "forge-search"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true

[dependencies]
anyhow = { workspace = true }
thiserror = { workspace = true }
```

**Add `"crates/forge-search"` to `[workspace] members` in ROOT `Cargo.toml`.**
**Add `forge-search = { path = "crates/forge-search" }` to ROOT `[workspace.dependencies]`.**

**`crates/forge-search/src/fuzzy.rs`:**
- `fuzzy_score(query: &str, candidate: &str) -> Option<f64>` — case-insensitive scan: for each query char, find it in candidate from last position forward. Score: consecutive match +10, word boundary (prev is `_`, `.`, `/`, or uppercase transition) +8, start of string +5. Gap penalty -1 per skipped char. Return None if not all chars found.
- `fuzzy_filter(query: &str, candidates: &[&str]) -> Vec<(usize, f64)>` — filter + sort desc by score, return (index, score) pairs
- Tests (≥3): exact match highest, partial scores, no match returns None

**`crates/forge-search/src/content.rs`:**
- `SearchOpts` struct: `regex: bool`, `case_sensitive: bool`, `whole_word: bool` — all `Default` false
- `SearchResult` struct: `file: String`, `line: usize`, `col: usize`, `text: String`
- `search_in_text(text: &str, file_name: &str, query: &str, opts: &SearchOpts) -> Vec<SearchResult>` — line-by-line search. If `case_sensitive` false, compare lowercase. If `whole_word`, check char boundaries. Return matching lines with line number and column.
- `search_directory(root: &std::path::Path, query: &str, opts: &SearchOpts) -> anyhow::Result<Vec<SearchResult>>` — walk directory recursively with `std::fs::read_dir`, skip hidden dirs (`.git`, `node_modules`, `target`), skip binary files (check first 512 bytes for null), read text files, call `search_in_text`.
- Tests (≥3): find in text, case insensitive matching, whole word boundary

**`crates/forge-search/src/lib.rs`:**
```rust
pub mod fuzzy;
pub mod content;
pub use fuzzy::{fuzzy_score, fuzzy_filter};
pub use content::{SearchOpts, SearchResult, search_in_text, search_directory};
```

---

## TASK 3: Git Status Panel

**Create `crates/forge-app/src/git_panel.rs`:**
- `FileStatus` enum: `Modified`, `Added`, `Deleted`, `Untracked`, `Renamed`, `Conflicted`
- Impl `Display` for `FileStatus` returning single-char: `M`, `A`, `D`, `?`, `R`, `!`
- `GitFile` struct: `path: String`, `status: FileStatus`
- `GitPanel` struct: `visible: bool`, `files: Vec<GitFile>`, `selected: usize`, `repo_path: Option<String>`
- `open()`, `close()`, `toggle()`
- `refresh(&mut self) -> anyhow::Result<()>` — if `repo_path` is Some, open repo with `git2::Repository::open()`, get statuses with `.statuses(None)`, map `git2::Status` flags to `FileStatus`, populate `self.files`
- `move_up()`, `move_down()`, `selected_file() -> Option<&GitFile>`
- `stage_file(&self, path: &str) -> anyhow::Result<()>` — open repo, get index, add path, write index
- `unstage_file(&self, path: &str) -> anyhow::Result<()>` — open repo, reset file to HEAD
- Tests (≥2): `GitFile` creation and `FileStatus::display()`, panel toggle works

**Add `git2 = { workspace = true }` to `crates/forge-app/Cargo.toml` under `[dependencies]`.**

## TASK 4: Git Gutter Diff Marks

**Create `crates/forge-app/src/git_gutter.rs`:**
- `DiffKind` enum: `Added`, `Modified`, `Deleted`
- `GutterMark` struct: `line: usize`, `kind: DiffKind`
- `compute_gutter_marks(old_text: &str, new_text: &str) -> Vec<GutterMark>` — compare line-by-line:
  - Build old lines vec and new lines vec
  - For each line in new: if line index >= old.len() → `Added`, if line != old[i] → `Modified`
  - For lines in old beyond new.len() → `Deleted` (mark at last new line)
- `gutter_color(kind: &DiffKind) -> [u8; 3]` — Added: `[13,188,121]` green, Modified: `[36,114,200]` blue, Deleted: `[205,49,49]` red
- Tests (≥2): added lines detected, modified lines detected

## TASK 5: Git Blame + Git Branches

**Create `crates/forge-app/src/git_blame.rs`:**
- `BlameLine` struct: `commit_hash: String`, `author: String`, `date: String`, `line_number: usize`
- `blame_file(repo_path: &str, file_path: &str) -> anyhow::Result<Vec<BlameLine>>` — open repo with `git2::Repository::open()`, call `.blame_file()`, iterate hunks, build `BlameLine` for each line
- Error handling: if repo or file doesn't exist, return descriptive error
- Tests (≥1): `BlameLine` struct creation compiles

**Create `crates/forge-app/src/git_branch.rs`:**
- `BranchInfo` struct: `name: String`, `is_current: bool`, `is_remote: bool`
- `list_branches(repo_path: &str) -> anyhow::Result<Vec<BranchInfo>>` — open repo, iterate `branches(None)`, build info
- `current_branch(repo_path: &str) -> anyhow::Result<Option<String>>` — open repo, check HEAD, get shorthand name
- `create_branch(repo_path: &str, name: &str) -> anyhow::Result<()>` — open repo, get HEAD commit, create branch from it
- Tests (≥2): `BranchInfo` struct creation, `current_branch` returns `None` for non-git dir (wrapped in error)

## TASK 6: Diff Viewer

**Create `crates/forge-app/src/diff_view.rs`:**
- `DiffLineKind` enum: `Added`, `Removed`, `Context`
- `DiffLine` struct: `kind: DiffLineKind`, `text: String`, `old_line_number: Option<usize>`, `new_line_number: Option<usize>`
- `DiffHunk` struct: `old_start: usize`, `new_start: usize`, `lines: Vec<DiffLine>`
- `compute_diff(old: &str, new: &str) -> Vec<DiffHunk>` — simple longest-common-subsequence diff:
  - Split into lines, build LCS table (or simple patience diff), generate hunks with 3 lines of context
  - Each hunk groups consecutive changes with surrounding context
- Tests (≥2): identical texts = no hunks, added lines create correct hunk

## TASK 7: Search Results Panel

**Add `forge-search = { workspace = true }` to `crates/forge-app/Cargo.toml`.**

**Create `crates/forge-app/src/search_panel.rs`:**
- Use `forge_search::{SearchOpts, SearchResult, search_in_text, search_directory}`.
- `SearchPanel` struct: `visible: bool`, `query: String`, `results: Vec<SearchResult>`, `selected: usize`, `opts: SearchOpts`
- `open()`, `close()`, `toggle()`
- `type_char(&mut self, c: char)` — append to query
- `backspace(&mut self)` — pop from query
- `clear(&mut self)` — clear query and results
- `search(&mut self, root: &std::path::Path)` — call `search_directory(root, &self.query, &self.opts)`, store results
- `search_in_buffer(&mut self, text: &str, file_name: &str)` — call `search_in_text(text, file_name, &self.query, &self.opts)`, store results
- `next_result(&mut self)`, `prev_result(&mut self)`, `selected_result() -> Option<&SearchResult>`
- `toggle_regex()`, `toggle_case()`, `toggle_whole_word()` — toggle opts fields
- Tests (≥2): type_char builds query, toggle methods flip opts

## TASK 8: Terminal UI Rendering

**Create `crates/forge-app/src/terminal_ui.rs`:**
- `RenderCommand` enum: `DrawChar { x: f32, y: f32, ch: char, fg: [u8;3], bg: [u8;3] }`, `DrawCursor { x: f32, y: f32 }`, `DrawRect { x: f32, y: f32, w: f32, h: f32, color: [u8;3] }`
- `TerminalUi` struct: `cell_width: f32`, `cell_height: f32`
- `TerminalUi::new(cell_width: f32, cell_height: f32) -> Self`
- `render(&self, grid: &forge_terminal::TerminalGrid, zone_x: f32, zone_y: f32) -> Vec<RenderCommand>` — iterate grid cells, create DrawChar for non-space cells, create DrawRect for cells with non-black background, create DrawCursor at cursor position
- `handle_key_input(key: char) -> String` — convert char to terminal-safe input string (special-case Enter → `\r\n`, Backspace → `\x7f`, Tab → `\t`, otherwise char as string)
- Tests (≥2): render produces commands for non-empty grid, handle_key_input maps correctly

**Add `forge-terminal = { path = "../forge-terminal" }` to `crates/forge-app/Cargo.toml`.**

## TASK 9: Go To Definition + References

**Create `crates/forge-app/src/go_to_def.rs`:**
- `Location` struct: `file: String`, `line: usize`, `col: usize`
- `NavStack` struct: `history: Vec<Location>`, `position: usize`
- `NavStack::new() -> Self`, `push(&mut self, loc: Location)`, `back(&mut self) -> Option<&Location>`, `forward(&mut self) -> Option<&Location>`
- `find_definition(symbol: &str, workspace_files: &[(String, String)]) -> Option<Location>` — for each (filename, content), search for `fn {symbol}`, `struct {symbol}`, `class {symbol}`, `def {symbol}`, `enum {symbol}` at word boundary; return first match with line and col
- Tests (≥3): find `fn` definition, NavStack back/forward cycle, not found returns None

**Create `crates/forge-app/src/references.rs`:**
- `find_references(symbol: &str, file_name: &str, text: &str) -> Vec<crate::go_to_def::Location>` — scan every line for whole-word occurrences of `symbol`, return location for each match
- `is_word_boundary(text: &str, start: usize, end: usize) -> bool` — check chars before/after position
- Tests (≥2): finds multiple references, excludes partial matches (`foobar` doesn't match `foo`)

## TASK 10: Symbol Outline + Workspace Symbols

**Create `crates/forge-app/src/outline_panel.rs`:**
- `SymbolKind` enum: `Function`, `Struct`, `Enum`, `Impl`, `Class`, `Method`, `Trait`, `Const`, `Module`
- `Symbol` struct: `name: String`, `kind: SymbolKind`, `line: usize`, `children: Vec<Symbol>`
- `extract_symbols(text: &str) -> Vec<Symbol>` — regex-based extraction:
  - `fn (\w+)` → Function
  - `struct (\w+)` → Struct
  - `enum (\w+)` → Enum
  - `impl (\w+)` → Impl
  - `trait (\w+)` → Trait
  - `class (\w+)` → Class (for Python/JS)
  - `def (\w+)` → Function (Python)
  - `const (\w+)` → Const
  - `mod (\w+)` → Module
- `OutlinePanel` struct: `visible: bool`, `symbols: Vec<Symbol>`, `selected: usize`
- `refresh(&mut self, text: &str)` — call `extract_symbols` and store
- `move_up()`, `move_down()`, `selected_symbol() -> Option<&Symbol>`
- Tests (≥2): extract Rust symbols (fn, struct, enum), extract Python symbols (def, class)

**Create `crates/forge-app/src/workspace_symbols.rs`:**
- `WorkspaceSymbol` struct: `name: String`, `kind: crate::outline_panel::SymbolKind`, `file: String`, `line: usize`
- `WorkspaceSymbolIndex` struct: `symbols: Vec<WorkspaceSymbol>`
- `WorkspaceSymbolIndex::build(files: &[(String, String)]) -> Self` — for each (filename, content), extract symbols, create WorkspaceSymbol entries
- `search(&self, query: &str) -> Vec<&WorkspaceSymbol>` — case-insensitive substring match on symbol name
- Tests (≥2): build index from file content, search finds matches

---

## AFTER ALL TASKS: Update `main.rs` Module Declarations

**Add these lines to `crates/forge-app/src/main.rs`** in the module declaration section (after the existing `mod` lines):

```rust
// Session 3 — Terminal + Git + Search
mod git_panel;
mod git_gutter;
mod git_blame;
mod git_branch;
mod diff_view;
mod search_panel;
mod terminal_ui;
mod go_to_def;
mod references;
mod outline_panel;
mod workspace_symbols;
```

---

## FINAL VERIFICATION (ALL MUST PASS)

```bash
cargo fmt --check        # Must exit 0
cargo clippy -- -D warnings  # Must exit 0 (in all crates)
cargo test --workspace   # Must exit 0
cargo check --workspace  # Must exit 0
```

**ALL FOUR must exit 0.** Do NOT modify `application.rs`. If any module/crate has a dependency issue, fix it. The workspace MUST compile cleanly.

**If a feature depends on `git2` and needs a real git repo for tests, use `tempdir` + `git2::Repository::init()` to create test repos.**
