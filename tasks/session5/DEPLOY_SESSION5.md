# SESSION 5 — AI Integration + Final Wiring + Polish
# ONE JULES TASK — Copy this ENTIRE file as one Jules prompt.
# PREREQUISITE: Session 4 must be merged first.
# THIS IS THE FINAL SESSION. application.rs IS MODIFIED HERE.
# ═══════════════════════════════════════════════════════════════

You are working on **Forge**, a GPU-accelerated code editor written in Rust. You will complete ALL 10 tasks below in sequence. Do them ALL.

**IMPORTANT: This session (and ONLY this session) is allowed to modify `crates/forge-app/src/application.rs`.** Earlier sessions created all the components — this session wires them together.

---

## RULES (MANDATORY)

1. **Rust 2021 edition.** No `.unwrap()` in production. `Result<T, E>` + `thiserror`. `.unwrap()` only in `#[cfg(test)]`.
2. **Every public function** MUST have ≥1 unit test.
3. `cargo fmt` + `cargo clippy -- -D warnings` = zero warnings.
4. UTF-8, LF. `///` for public API docs.
5. Add deps to ROOT `[workspace.dependencies]`, reference as `{ workspace = true }`.
6. When creating a new crate, add to `[workspace] members`.
7. **This session CAN AND MUST modify `application.rs` (Tasks 7-10).**

---

## TASK 1: AI Inline Completions

**Create `crates/forge-agent/src/inline_completion.rs`:**

- `InlineCompletion` struct: `text: String`, `line: usize`, `col: usize`, `ghost: bool`
- `InlineCompletionProvider` struct: `visible: bool`, `completion: Option<InlineCompletion>`, `debounce_ms: u64`
- `suggest(context: &str, line: usize, col: usize) -> Option<InlineCompletion>` — extract surrounding context (10 lines before, 5 after), build prompt, return suggestion
- `accept() -> Option<String>`, `dismiss()`, `is_visible() -> bool`
- For now this is a framework — the actual AI call (to Gemini etc.) is stubbed with a TODO comment. The logic for context extraction and accepting/dismissing is real.
- Tests (≥2): suggest creates completion with correct position, accept returns text and clears.

**Add `pub mod inline_completion;` to `crates/forge-agent/src/lib.rs`.**

## TASK 2: AI Chat Panel

**Create `crates/forge-agent/src/chat_panel.rs`:**

- `ChatRole` enum: `User`, `Assistant`, `System`
- `ChatMessage` struct: `role: ChatRole`, `content: String`, `timestamp: std::time::SystemTime`
- `ChatPanel` struct: `visible: bool`, `messages: Vec<ChatMessage>`, `input: String`, `model: String`
- `open()`, `close()`, `toggle()`, `type_char(c)`, `backspace()`, `send() -> Option<String>` — returns user message for processing
- `add_response(text: String)` — add assistant response
- `clear_history()`
- Tests (≥3): send returns message and clears input, messages accumulate, clear works.

## TASK 3: Markdown Preview + Image Preview

**Create `crates/forge-app/src/markdown_preview.rs`:**

- `MarkdownNode` enum: `Heading { level: u8, text: String }`, `Paragraph(String)`, `CodeBlock { language: String, code: String }`, `ListItem(String)`, `Bold(String)`, `Italic(String)`, `Link { text: String, url: String }`, `HorizontalRule`
- `parse_markdown(text: &str) -> Vec<MarkdownNode>` — basic parser: `#` headings, ` ``` ` code blocks, `**bold**`, `*italic*`, `[text](url)`, `- list items`, `---` rules
- Tests (≥3): headings parsed, code blocks parsed, bold/italic parsed.

**Create `crates/forge-app/src/image_preview.rs`:**

- `ImageFormat` enum: `Png`, `Jpeg`, `Gif`, `Svg`, `Unknown`
- `ImageInfo` struct: `format: ImageFormat`, `width: u32`, `height: u32`, `size_bytes: u64`
- `detect_format(path: &Path) -> ImageFormat` — check file extension and/or magic bytes
- `get_info(path: &Path) -> Result<ImageInfo>` — read dimensions from header bytes (basic: check PNG/JPEG headers)
- Tests (≥2): detect_format from extension, ImageInfo creation.

## TASK 4: Terminal Multiplexing

**Create `crates/forge-app/src/terminal_tabs.rs`:**

- `TerminalTab` struct: `id: u64`, `title: String`, `active: bool`
- `TerminalMultiplexer` struct: `tabs: Vec<TerminalTab>`, `active_tab: usize`, `next_id: u64`
- `create_tab(title: &str) -> u64` — add new terminal tab, return ID
- `close_tab(id: u64) -> bool`, `switch_to(id: u64)`, `active_id() -> Option<u64>`, `rename_tab(id: u64, title: &str)`
- Tests (≥3): create multiple tabs, switch between, close tab.

## TASK 5: Emmet Abbreviation Expansion

**Create `crates/forge-app/src/emmet.rs`:**

- `expand_abbreviation(abbr: &str) -> Result<String>` — basic Emmet support:
  - `div` → `<div></div>`
  - `div.class` → `<div class="class"></div>`
  - `div#id` → `<div id="id"></div>`
  - `div>span` → `<div><span></span></div>` (child)
  - `div+p` → `<div></div><p></p>` (sibling)
  - `ul>li*3` → `<ul><li></li><li></li><li></li></ul>` (multiply)
  - `div.foo#bar` → `<div class="foo" id="bar"></div>` (class + id)
  - Self-closing tags: `img`, `br`, `hr`, `input` → `<img />`
- Tests (≥5): basic tag, class, id, child, multiply.

## TASK 6: Accessibility Layer

**Create `crates/forge-app/src/accessibility.rs`:**

- `AriaRole` enum: `Editor`, `TreeView`, `TabList`, `Tab`, `Menu`, `MenuItem`, `Button`, `TextInput`, `StatusBar`, `Panel`
- `AccessibleElement` struct: `role: AriaRole`, `label: String`, `description: Option<String>`, `focused: bool`, `expanded: Option<bool>`
- `AccessibilityTree` struct: `elements: Vec<AccessibleElement>`, `focus_index: usize`
- `build_tree() -> Self` — create tree with standard editor elements
- `focus_next()`, `focus_prev()`, `get_focused() -> Option<&AccessibleElement>`
- `announce(message: &str)` — add to screen reader announcement queue (stored as Vec<String>)
- `announcements() -> &[String]`
- Tests (≥2): focus navigation cycles, announce stores messages.

## TASK 7: Wire Features into application.rs — Part 1 (Core)

**MODIFY `crates/forge-app/src/application.rs`:**

This is the main application file. Read its current contents first, then add the following integration:

1. Add fields to the main `Application` (or `App` or `ForgeApp` — use whatever the existing struct is called) struct:
   ```rust
   config: forge_config::ForgeConfig,
   theme_manager: forge_theme::ThemeManager,
   // keybindings: forge_keybindings::KeybindingResolver,  // Add if forge-keybindings exists
   ```

2. In the constructor/`new()` function:
   ```rust
   let config = forge_config::ForgeConfig::load().unwrap_or_default();
   let theme_manager = forge_theme::ThemeManager::new();
   ```

3. In the key event handler (wherever keyboard input is processed):
   - Check if Ctrl+Shift+P → toggle command palette visibility
   - Check if Ctrl+F → toggle find bar visibility
   - Check if Ctrl+H → toggle replace bar visibility
   - Check if Ctrl+` → toggle terminal
   - Check if Ctrl+, → toggle settings
   - Check if Ctrl+G → toggle go-to-line dialog

4. **Do NOT break existing functionality.** Add to existing match arms / if-else chains. Do NOT replace them.

**Update `crates/forge-app/Cargo.toml`** to add dependencies on: `forge-config`, `forge-theme`, `forge-keybindings` (all as `{ workspace = true }`). Only add if those crates exist in the workspace.

Tests: existing tests must still pass.

## TASK 8: Wire Features into application.rs — Part 2 (UI Panels)

**Continue modifying `crates/forge-app/src/application.rs`:**

Add these fields to the Application struct:
```rust
find_bar: crate::find_bar::FindBar,
command_palette: crate::command_palette::CommandPalette,
bottom_panel: crate::bottom_panel::BottomPanel,
notifications: crate::notifications::NotificationManager,
file_picker: crate::file_picker::FilePicker,
context_menu: crate::context_menu::ContextMenu,
```

Initialize all with their `Default` or `new()` constructors (add `#[derive(Default)]` to those structs if needed, or create constructor calls).

Wire keyboard shortcuts to panel toggles:
- Ctrl+Shift+P → `self.command_palette.open()` / `.close()`
- Ctrl+F → `self.find_bar.open()` / `.close()`
- Ctrl+P → `self.file_picker.open()` / `.close()`
- Escape → close whichever overlay is open (command_palette, find_bar, file_picker, context_menu)

**Do NOT break existing functionality.**

## TASK 9: Wire Features into application.rs — Part 3 (Terminal + Git)

**Continue modifying `crates/forge-app/src/application.rs`:**

Add:
```rust
// Only add these if the crates compile:
// terminal: Option<forge_terminal::Terminal>,
// git_panel: crate::git_panel::GitPanel,
```

Wire:
- Ctrl+` → toggle terminal in bottom panel (create terminal on first toggle if None)
- Status bar: show current git branch (call `crate::git_branch::current_branch()` if available)
- On file save: compute git gutter marks for the active file

**If any crate is missing or doesn't compile, wrap in conditional code or comment with TODO. The build MUST pass.**

## TASK 10: Final Integration + README Update

**Continue modifying `crates/forge-app/src/application.rs`:**

Add any remaining feature fields that exist:
- `split_layout: crate::split_editor::SplitLayout`
- `zen_mode: crate::zen_mode::ZenMode`

Wire:
- Ctrl+\ → split editor
- Ctrl+K, Ctrl+Z → enter/exit zen mode (or whatever key you choose)
- On startup: load config, apply theme, register default keybindings

**Update `README.md`** at the project root with:

```markdown
# Forge — GPU-Accelerated Code Editor

A high-performance code editor built in Rust with direct GPU rendering via wgpu.

## Features

- 🖥 GPU-rendered text with wgpu + glyphon
- 📝 Full-featured text editing (undo/redo, multi-cursor, find/replace)
- 🎨 7 built-in themes (Forge Dark, Forge Light, Monokai, Dracula, One Dark, Solarized, Nord)
- ⌨️ VS Code-compatible keybindings
- 🔍 Project-wide fuzzy search
- 📁 Multi-root workspace support
- 🖥 Integrated terminal with ANSI support
- 🌳 Git integration (status, blame, diff, branches)
- 🧩 Extension system (LSP, task runner)
- ♿ Accessibility layer with ARIA roles
- 🤖 AI integration framework (inline completions, chat)
- ⚡ Crash recovery and auto-save
- 📐 Code folding, indent guides, minimap
- 🔧 TOML-based configuration

## Building

```bash
cargo build --release
```

## Running

```bash
cargo run -p forge-app
```

## Testing

```bash
cargo test --workspace
```

## Architecture

Forge is organized as a Cargo workspace with the following crates:

| Crate | Description |
|-------|-------------|
| forge-core | Rope buffer, transactions, undo/redo, file I/O |
| forge-renderer | wgpu GPU pipeline, text atlas, rect renderer |
| forge-window | winit event loop, windowing |
| forge-app | Main application, UI components |
| forge-config | TOML configuration |
| forge-theme | Color themes engine |
| forge-input | Keyboard/mouse input, clipboard |
| forge-keybindings | Keyboard shortcut system |
| forge-types | Shared types (Color, Rect, Position) |
| forge-workspace | Multi-root workspace |
| forge-terminal | PTY, ANSI parser, grid buffer |
| forge-search | Fuzzy finder, content search |
| forge-lsp | Language Server Protocol client |
| forge-surfaces | UI surface manager |
| forge-agent | AI agent (chat, inline completions) |
| forge-net | Network/HTTP client |

## License

MIT OR Apache-2.0
```

---

## FINAL VERIFICATION (MOST IMPORTANT SESSION)

```bash
cargo fmt --check        # Must exit 0
cargo clippy -- -D warnings  # Must exit 0
cargo test --workspace   # Must exit 0
cargo check --workspace  # Must exit 0
cargo build --release    # Must succeed (this is the final build test)
```

**ALL FIVE must pass.** This is the final session — the editor should compile and run after this.

If any feature crate doesn't compile when integrated, **comment it out with a TODO** rather than leaving a broken build. The build MUST pass.
