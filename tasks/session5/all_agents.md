# Session 5 — AI + Integration + Polish (All 10 Agents)

> **Read `tasks/GLOBAL_RULES.md` first. This is the FINAL session. Agents 01-06 add features, Agents 07-10 WIRE EVERYTHING into application.rs.**

---

## Agent 01 — AI Inline Completions + Code Actions

### `crates/forge-agent/src/inline.rs`
`InlineCompletion { suggestion: Option<String>, position: Position, visible: bool }`. Ghost text rendering: dimmed/italic text after cursor. Tab accepts, Escape dismisses, Ctrl+Right accepts next word. `request_completion(context: &str) -> Result<String>` via `forge-net` HTTP client to configurable LLM endpoint (default: local Ollama `http://localhost:11434/api/generate`). Debounce 300ms.

### `crates/forge-agent/src/actions.rs`
`AiAction` enum: ExplainCode, RefactorSelection, GenerateTests, FixError, GenerateDocs, OptimizePerformance. `execute(action, selection, context) -> Result<String>`. Light bulb 💡 gutter indicator on lines with errors. Right-click menu: "AI: Explain", "AI: Refactor", "AI: Generate Tests". Results shown in AI chat panel or applied as edit.

---

## Agent 02 — AI Chat Panel Improvements

### `crates/forge-agent/src/chat_panel.rs`
Full chat UI (enhance existing `chat.rs`): `ChatPanel { messages: Vec<ChatMessage>, input: String, visible: bool, streaming: bool }`. `ChatMessage { role: Role, content: String, code_blocks: Vec<CodeBlock> }`. `CodeBlock { language: String, code: String }`. "Apply" button on code blocks → insert into editor. Context from current file auto-included. Streaming responses with partial rendering. Toggle with Ctrl+Shift+I.

---

## Agent 03 — Markdown Preview + Image Preview

### `crates/forge-app/src/markdown_preview.rs`
`MarkdownPreview { content: Vec<MdElement> }`. Parse markdown: headings (# → large bold text), **bold**, *italic*, `code`, ```code blocks```, - lists, > blockquotes, [links], ---. Render as styled glyphon text areas in split pane. Auto-update on edit (debounced 500ms). Ctrl+Shift+V toggle.

### `crates/forge-app/src/image_preview.rs`
`ImagePreview`. Detect image files (png/jpg/gif/webp/svg/ico by extension). On open: show image rendered as GPU texture instead of text editor. Zoom controls: Ctrl+= zoom in, Ctrl+- zoom out, Ctrl+0 fit. Display dimensions in status bar.

---

## Agent 04 — Terminal Multiplexer + Emmet

### `crates/forge-app/src/terminal_tabs.rs`
`TerminalManager { terminals: Vec<ManagedTerminal>, active: usize }`. `ManagedTerminal { terminal: Terminal, title: String, id: u32 }`. `create_terminal() -> u32`, `close_terminal(id)`, `rename_terminal(id, name)`, `split_terminal()`. Tab bar in bottom panel for switching. Ctrl+Shift+` creates new. Dropdown shows all terminals.

### `crates/forge-app/src/emmet.rs`
`EmmetEngine::expand(abbr: &str) -> Option<String>`. Support: `div.class` → `<div class="class"></div>`, `ul>li*3` → `<ul><li></li><li></li><li></li></ul>`, `div#id` → `<div id="id"></div>`, `p{text}` → `<p>text</p>`. Tab expansion in HTML/JSX files. Tests for 10+ common patterns.

---

## Agent 05 — Collaboration (CRDT) + Accessibility

### New crate `crates/forge-collab/`
Dependencies: `yrs = "0.21"`, `tokio`, `serde`, `anyhow`.

### `crates/forge-collab/src/crdt.rs`
`CollabDocument { doc: yrs::Doc, text: yrs::TextRef }`. `apply_local_edit(pos, delete_len, insert_text)` → generates CRDT update. `apply_remote_update(update: &[u8])`. `encode_state() -> Vec<u8>`, `merge_state(remote: &[u8])`. Tests for concurrent edits converging.

### `crates/forge-app/src/accessibility.rs`
`AccessibilityManager`. Tab-cycle focus between panels (editor → sidebar → bottom panel → status bar). `announce(text: &str)` for screen readers (prints to stdout as fallback). High contrast theme variant. Focus indicators: bright border on focused panel. Reduced motion mode: disable cursor blink, disable animations.

---

## Agent 06 — Performance Monitor + Installer Script

### `crates/forge-app/src/perf.rs`
`PerfMonitor { frame_budget_ms: f32, frame_times: RingBuffer, gc_pressure: f32 }`. Track: frame time distribution, GPU time, text layout time, parse time. Warn if frame >16ms. `report() -> PerfReport`. Optional debug overlay (F12 toggle) showing FPS, memory, GPU stats.

### `scripts/build_release.ps1`
PowerShell build script: `cargo build --release`, strip symbols, copy to `dist/forge.exe`, show binary size. Create portable zip. Optional: Inno Setup script `forge.iss` for Windows installer with desktop shortcut, "Open with Forge" right-click context menu, PATH registration, file associations (.rs, .js, .ts, .py, .go, .json, .toml, .md).

---

## Agent 07 — WIRE: Syntax Highlighting + File Tree into App

### MODIFY `crates/forge-app/src/application.rs`
This agent MODIFIES `application.rs` to connect Session 1 features:
1. **Syntax highlighting**: On file open, create `SyntaxParser` for detected language, parse buffer, get `HighlightSpan`s, map to glyphon `Attrs` with per-token colors instead of uniform white text.
2. **File tree**: Replace hardcoded sidebar string with real `FileTree::build_tree(workspace_root)`. Render visible nodes via `FileTreeUi`. Handle click events → open file in new tab via `TabManager`.
3. **TabManager**: Replace single `Editor` with `TabManager` managing multiple editors. Route all editor operations through `tab_manager.active_editor_mut()`.

---

## Agent 08 — WIRE: Terminal + Git + Bottom Panel into App

### MODIFY `crates/forge-app/src/application.rs`
Connect Session 2-3 features:
1. **Bottom panel**: Add `BottomPanel` to `AppState`. Ctrl+` toggles visibility. Render bottom panel background rects + tab indicators.
2. **Terminal**: Create `Terminal` on first bottom panel open. Route keyboard input to PTY when terminal focused. Render `TerminalGrid` via `TerminalUi`.
3. **Git**: On workspace open, detect git repo. Show branch in status bar. Compute gutter diff marks on file open/save. Git panel in sidebar (when activity bar "Git" icon clicked).
4. **Problems panel**: Wire diagnostics from `forge-lsp` (if available) to problems panel tab.

---

## Agent 09 — WIRE: Command Palette + Find/Replace + Overlays

### MODIFY `crates/forge-app/src/application.rs`
Connect overlay features:
1. **Command palette**: Ctrl+Shift+P opens `CommandPalette` overlay. Register all commands. Render on top of everything. Execute selected command.
2. **File picker**: Ctrl+P opens `FilePicker`. Scan workspace files, fuzzy filter. Enter opens file in tab.
3. **Find/replace**: Ctrl+F opens `FindBar` anchored to top of editor zone. Ctrl+H extends with replace. Match highlighting via `DecorationLayer`.
4. **Context menus**: Right-click in editor/tabs/file tree shows appropriate `ContextMenu`.
5. **Go to line**: Ctrl+G opens `GoToLine` overlay.

---

## Agent 10 — WIRE: Final Integration + README Update

### MODIFY `crates/forge-app/src/application.rs`
Final wiring:
1. **Keybindings**: Replace hardcoded key handling with `KeybindingResolver`. Load default keymap. Match key events → resolve command → dispatch.
2. **Config**: Load `ForgeConfig` on startup. Apply font size, theme, editor options.
3. **Theme**: Apply loaded theme colors to all UI elements (background, foreground, chrome).
4. **Notifications**: Show startup notification "Forge is ready". Wire notification rendering to bottom-right corner.
5. **Status segments**: Replace hardcoded status bar text with `StatusSegment` rendering.

### MODIFY `README.md`
Update README to reflect all implemented features, new crate list, full architecture diagram, build instructions, and usage guide.

### Run full verification:
```bash
cargo fmt --check
cargo clippy -- -D warnings  
cargo test --workspace
cargo build --release
```

---

**Acceptance**: Binary launches, opens file with syntax highlighting, file tree works, terminal works, git status shows, Ctrl+Shift+P opens palette. All tests pass.
