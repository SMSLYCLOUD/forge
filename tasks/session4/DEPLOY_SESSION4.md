# SESSION 4 — Advanced Editor + Extension System
# ONE JULES TASK — Copy this ENTIRE file as one Jules prompt.
# PREREQUISITE: Session 3 must be merged first.
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

## TASK 1: Split Editor Views

**Create `crates/forge-app/src/split_editor.rs`:**

- `SplitDirection` enum: `Horizontal`, `Vertical`
- `SplitNode` enum: `Leaf { editor_id: u64 }`, `Split { direction: SplitDirection, first: Box<SplitNode>, second: Box<SplitNode>, ratio: f32 }`
- `SplitLayout` struct: `root: SplitNode`, `active_editor: u64`, `next_id: u64`
- `SplitLayout::new() -> Self` — single leaf
- `split(direction) -> u64` — split active editor, returns new editor_id
- `close(editor_id) -> bool` — remove and collapse
- `set_active(editor_id)`
- `resize_split(ratio: f32)` — adjust the ratio of the active split
- `all_editor_ids() -> Vec<u64>` — collect all leaf editor IDs
- Tests (≥3): initial state has 1 editor, split creates 2, close returns to 1.

## TASK 2: Drag and Drop

**Create `crates/forge-app/src/drag_drop.rs`:**

- `DragKind` enum: `Tab { tab_id: usize }`, `File { path: String }`, `SplitBorder`
- `DragState` struct: `active: Option<DragKind>`, `start_x: f32`, `start_y: f32`, `current_x: f32`, `current_y: f32`
- `DragDrop` struct wrapping `DragState`
- `begin(kind, x, y)`, `update(x, y)`, `end() -> Option<DragKind>`, `cancel()`, `is_dragging() -> bool`, `delta() -> (f32, f32)`
- Tests (≥2): begin/end cycle, cancel clears state.

## TASK 3: Autocomplete

**Create `crates/forge-app/src/autocomplete.rs`:**

- `CompletionKind` enum: `Keyword`, `Function`, `Variable`, `Type`, `Snippet`, `File`, `Module`
- `CompletionItem` struct: `label: String`, `detail: Option<String>`, `kind: CompletionKind`, `insert_text: String`, `sort_priority: u32`
- `AutocompleteMenu` struct: `visible: bool`, `items: Vec<CompletionItem>`, `selected: usize`, `prefix: String`
- `show(items)`, `hide()`, `move_up()`, `move_down()`, `confirm() -> Option<&CompletionItem>`, `filter(prefix: &str)`
- `keyword_completions(language: &str) -> Vec<CompletionItem>` — builtin keywords for Rust, JavaScript, Python
- Tests (≥3): show/filter works, move_up/down cycles, confirm returns selected.

## TASK 4: Snippets

**Create `crates/forge-app/src/snippets.rs`:**

- `SnippetPlaceholder` struct: `index: u8`, `default: String`, `position: usize`
- `Snippet` struct: `prefix: String`, `body: String`, `description: String`, `placeholders: Vec<SnippetPlaceholder>`
- `SnippetEngine` struct: `snippets: Vec<Snippet>`
- `register(snippet)`, `find_by_prefix(prefix: &str) -> Vec<&Snippet>`, `expand(snippet: &Snippet) -> String` — replace `${1:default}` placeholders with defaults
- Built-in snippets for Rust: `fn`, `impl`, `test`, `struct`, `enum`, `match`, `if`, `for`, `while`, `loop`, `pub fn`
- Tests (≥3): find_by_prefix, expand replaces placeholders, Rust snippets registered.

## TASK 5: Settings UI

**Create `crates/forge-app/src/settings_ui.rs`:**

- `SettingsTab` enum: `Editor`, `Theme`, `Keybindings`, `Terminal`, `Git`, `Extensions`
- `SettingEntry` struct: `key: String`, `label: String`, `value: SettingValue`, `description: String`
- `SettingValue` enum: `Bool(bool)`, `Int(i64)`, `Float(f64)`, `Str(String)`, `Enum { options: Vec<String>, selected: usize }`
- `SettingsUI` struct: `visible: bool`, `active_tab: SettingsTab`, `entries: Vec<SettingEntry>`, `search_query: String`
- `open()`, `close()`, `set_tab(tab)`, `update_setting(key: &str, value: SettingValue) -> Result<()>`, `search(query: &str) -> Vec<&SettingEntry>`
- Tests (≥2): update_setting modifies value, search filters entries.

## TASK 6: Hover Info + Parameter Hints

**Create `crates/forge-app/src/hover.rs`:**

- `HoverInfo` struct: `content: String`, `range_start: (usize, usize)`, `range_end: (usize, usize)`, `visible: bool`
- `HoverProvider` struct
- `show(content, line, col)`, `hide()`, `is_visible() -> bool`
- `compute_hover(text: &str, line: usize, col: usize) -> Option<HoverInfo>` — extract word under cursor, search for `fn word`, `struct word` in text to build hover content
- Tests (≥2): hover on function shows signature, hover on empty = None.

**Create `crates/forge-app/src/param_hints.rs`:**

- `ParamHint` struct: `signature: String`, `parameters: Vec<String>`, `active_param: usize`
- `ParamHintProvider` struct: `visible: bool`, `hint: Option<ParamHint>`
- `show(signature, params, active_idx)`, `hide()`, `next_param()`, `prev_param()`
- `detect_function_call(text: &str, line: usize, col: usize) -> Option<String>` — scan backwards from cursor for `funcname(`
- Tests (≥2): show/hide works, detect_function_call finds name.

## TASK 7: Rename Symbol + Format on Save

**Create `crates/forge-app/src/rename.rs`:**

- `RenameInput` struct: `visible: bool`, `old_name: String`, `new_name: String`, `line: usize`, `col: usize`
- `RenameProvider` struct
- `open(old_name, line, col)`, `type_char(c)`, `backspace()`, `confirm() -> Option<RenameAction>`, `cancel()`
- `RenameAction` struct: `old_name: String`, `new_name: String`, `occurrences: Vec<(usize, usize)>` (line, col of each occurrence)
- `find_occurrences(text: &str, symbol: &str) -> Vec<(usize, usize)>` — whole-word search
- Tests (≥2): find_occurrences finds all, confirm returns action with new name.

**Create `crates/forge-app/src/formatter.rs`:**

- `Formatter` struct
- `format_on_save(text: &str, language: &str) -> Result<String>` — basic auto-formatting:
  - Fix inconsistent indentation (normalize to spaces or tabs based on config)
  - Remove trailing whitespace
  - Ensure final newline
- Tests (≥3): trailing whitespace removed, final newline added, indentation normalized.

## TASK 8: Zen Mode

**Create `crates/forge-app/src/zen_mode.rs`:**

- `ZenMode` struct: `active: bool`, `previous_state: ZenPreviousState`
- `ZenPreviousState` struct: `sidebar_visible: bool`, `statusbar_visible: bool`, `tabs_visible: bool`, `minimap_visible: bool`, `panel_visible: bool`
- `enter(state: ZenPreviousState) -> Self` — save current UI state, set active
- `exit(&self) -> ZenPreviousState` — return saved state for restoration
- `is_active() -> bool`
- Tests (≥2): enter/exit preserves state, is_active correct.

## TASK 9: Create forge-lsp Crate — LSP Client

**Create new crate `crates/forge-lsp/`.**

**`Cargo.toml`** deps: `serde`, `serde_json`, `anyhow`, `thiserror`, `tokio`, `futures`, `async-trait`.

**`src/protocol.rs`:**
- LSP message types as Rust structs:
  - `LspRequest` struct: `id: u64`, `method: String`, `params: serde_json::Value`
  - `LspResponse` struct: `id: u64`, `result: Option<serde_json::Value>`, `error: Option<LspError>`
  - `LspNotification` struct: `method: String`, `params: serde_json::Value`
  - `LspError` struct: `code: i32`, `message: String`
  - `Position` struct: `line: u32`, `character: u32`
  - `Range` struct: `start: Position`, `end: Position`
  - `Location` struct: `uri: String`, `range: Range`
  - `CompletionItem` struct: `label: String`, `kind: Option<u32>`, `detail: Option<String>`, `insert_text: Option<String>`
  - `Diagnostic` struct: `range: Range`, `severity: Option<u32>`, `message: String`, `source: Option<String>`
- Tests (≥2): serialize/deserialize round-trip for request and response.

**`src/client.rs`:**
- `LspClient` struct
- `async fn start(command: &str, args: &[&str]) -> Result<Self>` — spawn LSP server subprocess, set up stdin/stdout JSON-RPC
- `async fn initialize(root_uri: &str) -> Result<serde_json::Value>` — send initialize request
- `async fn shutdown() -> Result<()>`
- `async fn did_open(uri: &str, language: &str, text: &str) -> Result<()>`
- `async fn did_change(uri: &str, text: &str) -> Result<()>`
- `async fn completion(uri: &str, line: u32, character: u32) -> Result<Vec<CompletionItem>>`
- `async fn definition(uri: &str, line: u32, character: u32) -> Result<Option<Location>>`
- `async fn hover(uri: &str, line: u32, character: u32) -> Result<Option<String>>`
- `async fn references(uri: &str, line: u32, character: u32) -> Result<Vec<Location>>`
- `async fn formatting(uri: &str) -> Result<Vec<serde_json::Value>>`
- Tests (≥2): LspClient struct creation, protocol serialization.

**`src/lib.rs`:** re-export.

Add `"crates/forge-lsp"` to workspace members.

## TASK 10: Extension Manager UI + Task Runner

**Create `crates/forge-app/src/extension_panel.rs`:**
- `ExtensionInfo` struct: `id: String`, `name: String`, `version: String`, `description: String`, `enabled: bool`
- `ExtensionPanel` struct: `visible: bool`, `extensions: Vec<ExtensionInfo>`, `search_query: String`
- `open()`, `close()`, `search(query) -> Vec<&ExtensionInfo>`, `toggle_enabled(id: &str)`, `install(info: ExtensionInfo)`, `uninstall(id: &str)`
- Tests (≥2): install + search, toggle_enabled.

**Create `crates/forge-app/src/task_runner.rs`:**
- `TaskDef` struct: `name: String`, `command: String`, `args: Vec<String>`, `cwd: Option<String>`, `env: Vec<(String, String)>`
- `TaskStatus` enum: `Pending`, `Running`, `Success`, `Failed { code: i32, error: String }`
- `TaskRunner` struct: `tasks: Vec<TaskDef>`, `running: Option<(String, TaskStatus)>`
- `load_from_config(path: &Path) -> Result<Vec<TaskDef>>` — parse `forge-tasks.json`
- `run(name: &str) -> Result<()>`
- `status() -> Option<&TaskStatus>`
- Tests (≥2): load_from_config parses JSON, TaskDef creation.

---

## FINAL VERIFICATION

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test --workspace
cargo check --workspace
```

ALL FOUR must exit 0. **Do NOT modify `application.rs`.**
