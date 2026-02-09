# Session 4 — Advanced Editor + Extensions (All 10 Agents)

> **Read `tasks/GLOBAL_RULES.md` first. Each agent runs in parallel. Zero file conflicts.**

---

## Agent 01 — Split Editor Groups + Drag & Drop

### `crates/forge-app/src/editor_groups.rs`
`EditorGroup { id: u32, tabs: TabManager, zone: Zone }`. `EditorLayout { groups: Vec<EditorGroup>, active: u32, split_direction: SplitDir }`. `SplitDir`: Horizontal, Vertical. `split_right()` (Ctrl+\): clone active group layout, new empty group. `split_down()`. Max 4 groups. `focus_group(id)`, `close_group(id)`. Draggable borders for resize. Track active group for input routing.

### `crates/forge-app/src/drag_drop.rs`
`DragDrop { dragging: Option<DragPayload>, drop_indicator: Option<DropZone> }`. `DragPayload`: TabDrag(group_id, tab_idx), FileDrag(path). Start drag on mouse-down + move on tab. Show drop indicator line. On drop: move tab between groups or open file. `handle_mouse_move()`, `handle_mouse_up()`.

---

## Agent 02 — Autocomplete + Snippet Engine

### `crates/forge-app/src/autocomplete.rs`
`Autocomplete { visible: bool, items: Vec<CompletionItem>, selected: usize, trigger_pos: Position }`. `CompletionItem { label: String, kind: CompletionKind, detail: Option<String>, insert_text: String }`. `CompletionKind`: Function, Variable, Keyword, Snippet, Type, Module. Trigger on typing (3+ chars, or after `.` `::` `(`). Arrow navigation, Tab/Enter accept, Escape dismiss. Build items from: language keywords + symbols in current file (tree-sitter extracted names).

### `crates/forge-app/src/snippets.rs`
`SnippetEngine`. Parse VS Code JSON snippet format: `{ prefix, body, description }`. Tabstop `$1`, `$2`, placeholder `${1:default}`, variable `$TM_FILENAME`, `$CLIPBOARD`. `expand(prefix) -> Option<SnippetTemplate>`. `SnippetTemplate::render(vars) -> String`. Built-in snippets: `fn` (Rust function), `if`, `for`, `match`, `impl`, `struct`, `test`. Tab to cycle tabstops, Escape to finish.

---

## Agent 03 — Settings UI + Hover Info

### `crates/forge-app/src/settings_ui.rs`
`SettingsUi { visible: bool, search_query: String, categories: Vec<Category>, active_category: usize }`. `Category { name: String, settings: Vec<SettingEntry> }`. `SettingEntry { key: String, label: String, value: SettingValue, description: String }`. `SettingValue`: Bool(bool), String(String), Number(f64), Enum(Vec<String>, usize). Render as scrollable list with toggle switches, dropdowns, text inputs. Ctrl+, keybinding. Reads/writes `forge-config`.

### `crates/forge-app/src/hover_info.rs`
`HoverProvider::provide(buffer, tree, position) -> Option<HoverContent>`. `HoverContent { text: String, range: (usize, usize) }`. Use tree-sitter: find node at position, get parent (for function signature), extract doc comments (lines starting with `///` or `#` above). Render as floating tooltip after 500ms hover delay. Dismiss on mouse move.

---

## Agent 04 — Parameter Hints + Rename Symbol

### `crates/forge-app/src/param_hints.rs`
`ParamHintProvider::provide(buffer, tree, position) -> Option<ParamHint>`. `ParamHint { signature: String, active_param: usize, params: Vec<String> }`. Trigger on `(` and `,`. Use tree-sitter to find enclosing `call_expression`, locate function definition, extract parameters. Render as floating tooltip above cursor showing `fn_name(param1, **param2**, param3)` with active param bolded.

### `crates/forge-app/src/rename_symbol.rs`
`RenameProvider`. F2 keybinding opens inline input at cursor. `prepare_rename(buffer, pos) -> Option<String>` returns current symbol name. `apply_rename(buffer, old_name, new_name) -> Transaction`. Find all occurrences of `old_name` in current file (word-boundary aware), create Transaction with all replacements. Single undo reverts all.

---

## Agent 05 — Formatter + Zen Mode

### `crates/forge-app/src/formatter.rs`
`Formatter::format(file_path: &str, content: &str, lang: Language) -> Result<String>`. Shell out to: `rustfmt` (Rust), `prettier` (JS/TS/HTML/CSS/JSON), `black` (Python), `gofmt` (Go). Detect formatter in PATH. `format_on_save: bool` config option. Apply diff-based: compute edit operations from old → new to preserve cursor position. Shift+Alt+F keybinding.

### `crates/forge-app/src/zen_mode.rs`
`ZenMode { active: bool, saved_layout: Option<LayoutConfig> }`. `enter()`: hide sidebar, status bar, tab bar, activity bar, bottom panel; center editor with max-width 120 chars; full-screen window. `exit()`: restore saved layout. `toggle()`. Ctrl+K Z keybinding. Escape also exits.

---

## Agent 06 — forge-lsp: Server + Transport

### New crate `crates/forge-lsp/`
Dependencies: `tokio`, `serde`, `serde_json`, `anyhow`, `thiserror`, `tracing`, `lsp-types = "0.97"`.

### `crates/forge-lsp/src/server.rs`
`LspServer { process: Child, stdin: ChildStdin, stdout: BufReader<ChildStdout> }`. `spawn(command: &str, args: &[&str]) -> Result<Self>`. Server registry: `rust-analyzer` for .rs, `typescript-language-server` for .ts/.js, `pyright` for .py. Auto-detect in PATH. `kill()`, `is_alive()`.

### `crates/forge-lsp/src/transport.rs`
`Transport`. `send(msg: &JsonValue) -> Result<()>`: write `Content-Length: N\r\n\r\n{json}`. `receive() -> Result<JsonValue>`: read `Content-Length` header, read N bytes, parse JSON. Async via tokio. Request/response ID matching with `PendingRequests` map.

---

## Agent 07 — forge-lsp: Client API + Diagnostics

### `crates/forge-lsp/src/client.rs`
`LspClient { server: LspServer, capabilities: ServerCapabilities, next_id: i64 }`. Methods: `initialize(root_uri)`, `did_open(uri, text, lang)`, `did_change(uri, changes)`, `completion(uri, pos) -> Vec<CompletionItem>`, `hover(uri, pos) -> Option<Hover>`, `definition(uri, pos) -> Option<Location>`, `references(uri, pos) -> Vec<Location>`, `formatting(uri) -> Vec<TextEdit>`. Each wraps JSON-RPC request/response.

### `crates/forge-lsp/src/diagnostics.rs`
`DiagnosticHandler`. On `textDocument/publishDiagnostics` notification: map `lsp_types::Diagnostic` to `forge-renderer::Decoration` (wavy underlines). Map severity to colors: Error=red, Warning=yellow, Info=blue, Hint=gray. Update `ProblemsPanel` from Session 2. Count errors/warnings for status bar.

### `crates/forge-lsp/src/lib.rs`
Re-export all public types.

---

## Agent 08 — forge-plugin: WASM Runtime + Host API

### New crate `crates/forge-plugin/`
Dependencies: `wasmtime = "27"`, `anyhow`, `serde`, `serde_json`.

### `crates/forge-plugin/src/runtime.rs`
`PluginRuntime { engine: wasmtime::Engine, store: wasmtime::Store<PluginState> }`. `load_plugin(path: &Path) -> Result<Plugin>`. `Plugin { instance: wasmtime::Instance, name: String }`. Memory limit: 64MB. CPU: fuel-based limiting (1M fuel per call). `call(func_name, args) -> Result<Vec<Value>>`. `unload()`.

### `crates/forge-plugin/src/host_api.rs`
Expose to WASM plugins via linker: `forge_read_buffer() -> String`, `forge_insert_text(text: &str)`, `forge_register_command(name: &str)`, `forge_show_notification(msg: &str, level: i32)`, `forge_get_config(key: &str) -> String`. Capability-based: plugins declare required permissions in manifest.

---

## Agent 09 — Debugger (DAP Client) + Debug UI

### New crate `crates/forge-debug/`
Dependencies: `tokio`, `serde`, `serde_json`, `anyhow`.

### `crates/forge-debug/src/client.rs`
`DebugClient { transport: DapTransport }`. DAP protocol over stdio. `launch(program, args)`, `set_breakpoints(file, lines)`, `continue_()`, `step_over()`, `step_into()`, `step_out()`, `pause()`, `disconnect()`. Parse responses: stopped events, variable scopes, stack frames.

### `crates/forge-app/src/debug_ui.rs`
Debug panel in bottom area: breakpoint gutter indicators (red dots), variable inspector tree view, call stack list, watch expressions, debug toolbar with Continue/Step Over/Step Into/Step Out/Restart/Stop buttons. F5 = start debug, F9 = toggle breakpoint, F10 = step over, F11 = step into.

---

## Agent 10 — Task Runner + Extension Panel UI

### `crates/forge-app/src/task_runner.rs`
`TaskRunner`. Read `forge-tasks.json`: `{ tasks: [{ label, command, group, problemMatcher }] }`. `run_task(label)`: spawn command in terminal, capture output, parse errors via problem matcher regex. Built-in tasks: `cargo build`, `cargo test`, `cargo clippy`. Ctrl+Shift+B = run build task.

### `crates/forge-app/src/extensions_panel.rs`
`ExtensionsPanel { visible: bool, installed: Vec<ExtensionInfo>, search_query: String }`. `ExtensionInfo { name, version, description, enabled: bool }`. List installed WASM plugins from `~/.config/forge/extensions/`. Enable/disable toggle. Uninstall button (deletes .wasm file). Search placeholder (local only for now).

---

**Acceptance**: `cargo check --workspace && cargo test --workspace` passes.
