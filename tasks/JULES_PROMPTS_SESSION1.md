# Jules Session 1 — Copy-Paste Prompts (10 Agents)

> **Instructions**: Open 10 Jules sessions on repo `SMSLYCLOUD/forge` (master branch). Paste ONE prompt per session. All 10 run in parallel — they touch different files so no conflicts.

---

## AGENT 1 — Fix Compiler Warnings + Create forge-types Crate

```
You are working on the Forge editor, a Rust workspace with 19 crates at the root of this repo.

Follow these steps IN ORDER. Do not skip any step. Do not proceed to the next step until the current step compiles.

STEP 1: Read the file tasks/GLOBAL_RULES.md to understand the project rules.

STEP 2: Read the file tasks/session1/agent_01.md for full details on your task.

STEP 3: Run `cargo check --workspace 2>&1` and note every compiler warning.

STEP 4: Fix EVERY warning you found in step 3. For each warning:
  - If it says "unused import" → remove the import
  - If it says "unused variable" → prefix with underscore: `_var_name`
  - If it says "dead code" or "never read" → add `#[allow(dead_code)]` with a `// TODO: will be used later` comment above it
  - If it says "unused mut" → remove the `mut` keyword

STEP 5: Run `cargo check --workspace 2>&1` again. If there are still warnings, go back to step 4. Repeat until ZERO warnings.

STEP 6: Create the directory crates/forge-types/src/

STEP 7: Create the file crates/forge-types/Cargo.toml with this exact content:
[package]
name = "forge-types"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { workspace = true }

STEP 8: Create the file crates/forge-types/src/lib.rs with shared types: Color (rgba, rgb, to_u8_array), Rect (x, y, width, height, contains), Position (line, col), Size (width, height). All should derive Debug, Clone, Copy, PartialEq, Serialize, Deserialize. Add unit tests for Color::to_u8_array and Rect::contains.

STEP 9: Open the root Cargo.toml. Add "crates/forge-types" to the [workspace] members list. Add `forge-types = { path = "crates/forge-types" }` under [workspace.dependencies].

STEP 10: Run `cargo check --workspace`. Fix any errors.

STEP 11: Run `cargo test -p forge-types`. All tests must pass.

STEP 12: Run `cargo clippy -- -D warnings`. Fix any warnings.

STEP 13: Run `cargo fmt`.

DONE. All of cargo check, cargo test, cargo clippy, and cargo fmt must succeed with zero errors and zero warnings.
```

---

## AGENT 2 — Implement forge-config + forge-keybindings

```
You are working on the Forge editor, a Rust workspace with 19 crates at the root of this repo.

Follow these steps IN ORDER. Do not skip any step.

STEP 1: Read tasks/GLOBAL_RULES.md for project rules.

STEP 2: Read tasks/session1/agent_02.md for full code details.

STEP 3: Open the existing file crates/forge-config/src/lib.rs. You will REPLACE its contents with a full TOML configuration system.

STEP 4: Create the file crates/forge-config/src/editor.rs with EditorConfig struct containing: tab_size (usize, default 4), insert_spaces (bool, default true), word_wrap (bool, default false), line_numbers (bool, default true), minimap (bool, default true), auto_save_delay_ms (u64, default 30000), cursor_blink (bool, default true). Derive Serialize, Deserialize. Implement Default.

STEP 5: Create the file crates/forge-config/src/terminal.rs with TerminalConfig struct containing: shell (Option<String>), scrollback (usize, default 10000), cursor_style (String, default "block"), font_size (f32, default 13.0). Derive Serialize, Deserialize. Implement Default.

STEP 6: Replace crates/forge-config/src/lib.rs with ForgeConfig struct that contains editor: EditorConfig, terminal: TerminalConfig, theme: String (default "Forge Dark"), font_family: String (default "Cascadia Code"), font_size: f32 (default 14.0). Add load() that reads from ~/.config/forge/config.toml (use dirs-next crate), save() that writes TOML, and config_path(). Use #[serde(default)] so missing fields use defaults. Add `mod editor; mod terminal;` and re-export both.

STEP 7: Add `dirs-next = { workspace = true }` and `toml = { workspace = true }` to crates/forge-config/Cargo.toml dependencies if not already there. Also add `anyhow = { workspace = true }` and `serde = { workspace = true }`.

STEP 8: Run `cargo check -p forge-config`. Fix any errors before proceeding.

STEP 9: Add tests to forge-config: test that Default creates valid config, test that save() then load() round-trips correctly using a temp directory.

STEP 10: Run `cargo test -p forge-config`. All tests must pass.

STEP 11: Create directory crates/forge-keybindings/src/

STEP 12: Create crates/forge-keybindings/Cargo.toml:
[package]
name = "forge-keybindings"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { workspace = true }
serde_json = { workspace = true }
anyhow = { workspace = true }

STEP 13: Create crates/forge-keybindings/src/lib.rs with:
- KeyCombo struct { ctrl: bool, shift: bool, alt: bool, key: String } — derive Hash, Eq
- Keybinding struct { key: KeyCombo, command: String, when: Option<String> }
- KeybindingResolver struct with HashMap index for fast lookup
- KeybindingResolver::new(bindings) builds the index
- KeybindingResolver::resolve(&self, combo) -> Option<&str> returns last matching command (later bindings override earlier)
- KeybindingResolver::default_keymap() returns Vec<Keybinding> with: Ctrl+S=file.save, Ctrl+Z=edit.undo, Ctrl+Y=edit.redo, Ctrl+F=edit.find, Ctrl+H=edit.replace, Ctrl+Shift+P=command_palette, Ctrl+P=file_picker, Ctrl+W=tab.close, Ctrl+G=go.line, F12=go.definition
- Add tests: resolve Ctrl+S returns "file.save", override binding takes precedence

STEP 14: Add "crates/forge-keybindings" to [workspace] members in root Cargo.toml.

STEP 15: Run `cargo check --workspace`. Fix any errors.

STEP 16: Run `cargo test -p forge-config -p forge-keybindings`. All tests must pass.

STEP 17: Run `cargo clippy -- -D warnings`. Fix any clippy warnings.

STEP 18: Run `cargo fmt`.

DONE.
```

---

## AGENT 3 — forge-theme + forge-icons

```
You are working on the Forge editor, a Rust workspace at the root of this repo.

Follow these steps IN ORDER.

STEP 1: Read tasks/GLOBAL_RULES.md and tasks/session1/agent_03.md.

STEP 2: Create crates/forge-theme/src/token.rs with TokenColor struct: scope (Vec<String>), settings (TokenSettings). TokenSettings: foreground (Option<String>), font_style (Option<String>). Derive Serialize, Deserialize.

STEP 3: Create crates/forge-theme/src/builtin.rs with two functions:
- forge_dark() -> Theme: returns Theme with name "Forge Dark", kind Dark, and a HashMap of 50+ VS Code color keys like "editor.background" => "#1a1b26", "editor.foreground" => "#d4d4d4", "activityBar.background" => "#1e1e2e", "sideBar.background" => "#181825", "statusBar.background" => "#007acc", "tab.activeBackground" => "#1e1e2e", "tab.inactiveBackground" => "#2d2d2d", etc. Also add a Vec of TokenColor entries for keyword (pink #ff79c6), function (green #50fa7b), type (cyan #8be9fd), string (yellow #f1fa8c), number (purple #bd93f9), comment (gray #6272a4).
- forge_light() -> Theme: similar but light colors.

STEP 4: Replace crates/forge-theme/src/lib.rs with:
- mod builtin; mod token; pub use token::TokenColor;
- Theme struct { name: String, kind: ThemeKind, colors: HashMap<String, String>, token_colors: Vec<TokenColor> } with Serialize, Deserialize
- ThemeKind enum { Dark, Light, HighContrast } with Default = Dark
- Theme::color(&self, key) -> Option<[f32; 4]> that looks up key in colors map and converts hex to float array
- Theme::default_dark() and default_light() calling builtin functions
- parse_hex_color(hex: &str) -> Option<[f32; 4]> handling 6-digit and 8-digit hex with # prefix
- Tests: parse "#ff8000" works, default_dark() has non-empty colors

STEP 5: Make sure crates/forge-theme/Cargo.toml has serde and serde_json as dependencies.

STEP 6: Run `cargo check -p forge-theme`. Fix errors.

STEP 7: Run `cargo test -p forge-theme`. Tests pass.

STEP 8: Create directory crates/forge-icons/src/

STEP 9: Create crates/forge-icons/Cargo.toml (name = "forge-icons", version/edition workspace, no dependencies needed).

STEP 10: Create crates/forge-icons/src/lib.rs with:
- FileIcon enum: Rust, JavaScript, TypeScript, Python, Go, C, Cpp, Json, Toml, Yaml, Html, Css, Markdown, Shell, Docker, Git, Generic
- FileIcon::from_extension(ext: &str) -> Self mapping file extensions
- FileIcon::glyph(&self) -> &'static str returning emoji for each type
- UiIcon enum: Folder, FolderOpen, Search, Settings, Git, Debug, Extensions, Terminal with glyph() method
- Tests: from_extension("rs") == Rust, from_extension("xyz") == Generic

STEP 11: Add "crates/forge-icons" to [workspace] members in root Cargo.toml.

STEP 12: Run `cargo check --workspace && cargo test -p forge-theme -p forge-icons`.

STEP 13: Run `cargo clippy -- -D warnings` and `cargo fmt`.

DONE.
```

---

## AGENT 4 — Buffer Tests + Clipboard + Recovery

```
You are working on the Forge editor, a Rust workspace at the root of this repo.

Follow these steps IN ORDER.

STEP 1: Read tasks/GLOBAL_RULES.md and tasks/session1/agent_04.md.

STEP 2: Open crates/forge-core/src/buffer.rs. Read the existing code to understand the Buffer API — what methods exist, what their signatures are.

STEP 3: Add a new test module at the bottom of buffer.rs (or in an existing #[cfg(test)] block). Add edge case tests using the ACTUAL method names from step 2. Include tests for:
- Empty buffer: creating new buffer has 1 line
- Deleting from empty buffer doesn't panic
- Inserting emoji "👋🌍" works correctly
- Inserting CJK characters "你好世界" preserves string length
- Large buffer: create string with 100,000 lines, verify line count
- If there's CRLF handling, test that "\r\n" is normalized

IMPORTANT: Use only methods that actually exist on Buffer. Do NOT write tests for methods that don't exist. Read the code first.

STEP 4: Run `cargo test -p forge-core`. Fix any test failures by adjusting test code to match actual API.

STEP 5: Create crates/forge-input/src/clipboard.rs with a Clipboard struct wrapping arboard::Clipboard. Methods: new() -> Result<Self>, copy(&mut self, text: &str) -> Result<()>, paste(&mut self) -> Result<String>. Use anyhow for errors.

STEP 6: Open crates/forge-input/src/lib.rs and add `pub mod clipboard;`.

STEP 7: Make sure crates/forge-input/Cargo.toml has `arboard = { workspace = true }` and `anyhow = { workspace = true }` in dependencies.

STEP 8: Run `cargo check -p forge-input`. Fix errors.

STEP 9: Create crates/forge-core/src/recovery.rs with RecoveryManager struct. It uses dirs-next to find data dir, saves buffer content to ~/.local/share/forge/recovery/ (or AppData on Windows) with a hashed filename, and can recover or clear saved content. Add tests using temp directories.

STEP 10: Open crates/forge-core/src/lib.rs and add `pub mod recovery;`.

STEP 11: Make sure forge-core Cargo.toml has `dirs-next = { workspace = true }` in dependencies.

STEP 12: Run `cargo check --workspace && cargo test -p forge-core -p forge-input`.

STEP 13: Run `cargo clippy -- -D warnings` and `cargo fmt`.

DONE.
```

---

## AGENT 5 — forge-syntax: Tree-sitter Parser + Language Detection

```
You are working on the Forge editor, a Rust workspace at the root of this repo.

Follow these steps IN ORDER. This creates a NEW crate.

STEP 1: Read tasks/GLOBAL_RULES.md and tasks/session1/agent_05.md.

STEP 2: Check crates.io for the latest versions of: tree-sitter, tree-sitter-rust, tree-sitter-javascript, tree-sitter-python, tree-sitter-json, tree-sitter-toml-ng. NOTE: some tree-sitter grammar crates may have different names or version requirements. Search crates.io to find the correct crate names and compatible versions.

STEP 3: Add ALL tree-sitter dependencies to root Cargo.toml under [workspace.dependencies]. If a crate doesn't exist with that exact name, find the correct alternative on crates.io.

STEP 4: Add "crates/forge-syntax" to [workspace] members in root Cargo.toml.

STEP 5: Create crates/forge-syntax/Cargo.toml referencing the workspace dependencies:
[package]
name = "forge-syntax"
version = "0.1.0"
edition = "2021"

[dependencies]
tree-sitter = { workspace = true }
tree-sitter-rust = { workspace = true }
tree-sitter-javascript = { workspace = true }
tree-sitter-python = { workspace = true }
tree-sitter-json = { workspace = true }
(add others as found in step 2)
anyhow = { workspace = true }
thiserror = { workspace = true }

STEP 6: Create crates/forge-syntax/src/language.rs with Language enum (Rust, JavaScript, TypeScript, Python, Go, C, Cpp, Json, Toml, Yaml, Html, Css, Markdown, Shell, Unknown). Add from_extension(ext) and from_path(path). Add tree_sitter_language(&self) -> Option<tree_sitter::Language> that returns the grammar for supported languages. Check tree-sitter grammar crate APIs — some use LANGUAGE constant, others use language() function.

STEP 7: Run `cargo check -p forge-syntax`. This is the critical step — tree-sitter grammar APIs vary between crates. Fix any compile errors by checking how each grammar crate exports its language. Common patterns:
  - `tree_sitter_rust::LANGUAGE.into()`
  - `tree_sitter_rust::language()` 
  - Check the crate docs if unsure.

STEP 8: Create crates/forge-syntax/src/parser.rs with SyntaxParser { parser, language }. Methods: new(lang), parse(text) -> Result<Tree>, reparse(text, old_tree) -> Result<Tree>.

STEP 9: Create crates/forge-syntax/src/lib.rs: pub mod language; pub mod parser; pub use both.

STEP 10: Run `cargo check -p forge-syntax`. Fix errors.

STEP 11: Add tests: parse "fn main() {}" with Rust language, verify tree root node is "source_file". Test Language::from_extension("rs") == Rust. Test from_extension("unknown") == Unknown.

STEP 12: Run `cargo test -p forge-syntax`. All pass.

STEP 13: Run `cargo clippy -- -D warnings` and `cargo fmt`.

DONE.
```

---

## AGENT 6 — Syntax Highlighter + Token Colors

```
You are working on the Forge editor, a Rust workspace at the root of this repo.

Follow these steps IN ORDER.

STEP 1: Read tasks/GLOBAL_RULES.md and tasks/session1/agent_06.md.

STEP 2: Check if crates/forge-syntax/ exists. If it does NOT exist, you must create it first:
  - Follow ALL steps from tasks/session1/agent_05.md to create the forge-syntax crate with language detection and parser.
  - Run `cargo check -p forge-syntax` to confirm it works.
  - Then continue with step 3.

STEP 3: Create crates/forge-syntax/src/highlighter.rs with:
- TokenType enum: Keyword, Function, Type, String, Number, Comment, Operator, Punctuation, Variable, Constant, Namespace, Property, Parameter, Macro, Attribute, Label, Builtin, Plain
- HighlightSpan struct { start_byte: usize, end_byte: usize, token_type: TokenType }
- Highlighter struct with highlight(tree, source_bytes, lang) -> Vec<HighlightSpan>
- Walk tree-sitter CST recursively. For leaf nodes (child_count == 0), classify by node kind:
  - "line_comment" | "block_comment" | "comment" => Comment
  - "string_literal" | "string" | "raw_string_literal" => String
  - "integer_literal" | "float_literal" | "number" => Number
  - "fn" | "let" | "mut" | "pub" | "use" | "struct" | "if" | "else" | "return" etc. => Keyword
  - Punctuation characters => Punctuation
  - Operators => Operator
  - Everything else => Plain

STEP 4: Run `cargo check -p forge-syntax`. Fix errors.

STEP 5: Create crates/forge-syntax/src/colors.rs with default_color(token: TokenType) -> [u8; 3] returning Dracula-style colors: Keyword=[255,121,198], Function=[80,250,123], Type=[139,233,253], String=[241,250,140], Number=[189,147,249], Comment=[98,114,164], Operator=[255,184,108], Variable/Plain=[248,248,242].

STEP 6: Update crates/forge-syntax/src/lib.rs to add: pub mod highlighter; pub mod colors; and re-export Highlighter, HighlightSpan, TokenType.

STEP 7: Run `cargo check -p forge-syntax`.

STEP 8: Add test: parse "fn main() { let x = 42; }" with Rust, call Highlighter::highlight, verify spans contain at least one Keyword and one Number token type.

STEP 9: Run `cargo test -p forge-syntax`. All pass.

STEP 10: Run `cargo clippy -- -D warnings` and `cargo fmt`.

DONE.
```

---

## AGENT 7 — Real File Tree + File Tree UI

```
You are working on the Forge editor, a Rust workspace at the root of this repo.

Follow these steps IN ORDER.

STEP 1: Read tasks/GLOBAL_RULES.md and tasks/session1/agent_07.md.

STEP 2: Open crates/forge-surfaces/src/file_explorer.rs. Read whatever is there currently.

STEP 3: REPLACE the contents of crates/forge-surfaces/src/file_explorer.rs with a real file tree implementation:
- FileNode struct { name: String, path: PathBuf, kind: NodeKind, children: Vec<FileNode>, expanded: bool, depth: usize }
- NodeKind enum { File, Directory }
- FileNode::build_tree(root: &Path, max_depth: usize) -> Result<Self> that recursively walks directories
- Skip hidden files (starting with '.'), skip "target" and "node_modules" directories
- Sort children: directories first, then alphabetically case-insensitive
- FileNode::toggle(target_path) -> bool to expand/collapse
- FileNode::flatten_visible() -> Vec<&FileNode> that returns all visible nodes (expanded dirs show their children)

STEP 4: Make sure crates/forge-surfaces/Cargo.toml has `anyhow = { workspace = true }` in dependencies.

STEP 5: Run `cargo check -p forge-surfaces`. Fix errors.

STEP 6: Add tests to file_explorer.rs:
- Create a temp directory with some files/folders, build_tree, verify structure
- Test toggle: expand a directory, flatten, verify children are visible

STEP 7: Run `cargo test -p forge-surfaces`. All pass.

STEP 8: Now open crates/forge-app/src/main.rs. Look at how it declares modules (mod statements at the top).

STEP 9: Create crates/forge-app/src/file_tree_ui.rs with FileTreeUi struct that has scroll_offset, selected_index, hovered_index fields. It should have a render_rects() method that returns rectangles for hover highlight and selection highlight. Also create a DisplayNode struct { label, depth, is_dir, expanded }.

STEP 10: Add `mod file_tree_ui;` to main.rs alongside the other mod declarations.

STEP 11: Run `cargo check -p forge-app`. Fix errors — make sure the Rect type matches what forge-app uses (check rect_renderer.rs for the Rect struct definition and use that exact type).

STEP 12: Run `cargo clippy -- -D warnings` and `cargo fmt`.

DONE.
```

---

## AGENT 8 — Tab Manager + File I/O

```
You are working on the Forge editor, a Rust workspace at the root of this repo.

Follow these steps IN ORDER.

STEP 1: Read tasks/GLOBAL_RULES.md and tasks/session1/agent_08.md.

STEP 2: Open crates/forge-app/src/editor.rs. Read how Editor is structured — what fields it has, how Editor::new() and Editor::open_file() work. You need this to create TabManager.

STEP 3: Create crates/forge-app/src/tab_manager.rs with:
- Tab struct { title: String, path: Option<PathBuf>, editor: Editor, is_modified: bool }
- TabManager struct { tabs: Vec<Tab>, active: usize }
- TabManager::new() -> Self (empty tabs)
- open_file(&mut self, path: &str) -> Result<()>: check for duplicate paths first, if already open just switch to it, otherwise create new Tab with Editor::open_file()
- close_tab(&mut self, idx): remove tab, adjust active index
- close_current(&mut self): close active tab
- next_tab(&mut self): cycle active forward
- prev_tab(&mut self): cycle active backward
- active_editor(&self) -> Option<&Editor>
- active_editor_mut(&mut self) -> Option<&mut Editor>
- tab_count(&self) -> usize

STEP 4: Add `mod tab_manager;` to crates/forge-app/src/main.rs.

STEP 5: Run `cargo check -p forge-app`. Fix errors — make sure you import the right Editor type and use the correct API.

STEP 6: Create crates/forge-core/src/file_io.rs with:
- FileIO struct (unit struct)
- save_atomic(path: &Path, content: &str) -> Result<()>: write to path.with_extension("forge-tmp"), then std::fs::rename to path
- is_binary(path: &Path) -> Result<bool>: read file, check first 8192 bytes for null byte (0x00)
- detect_line_ending(content: &str) -> &'static str: if contains "\r\n" return "\r\n" else "\n"

STEP 7: Add `pub mod file_io;` to crates/forge-core/src/lib.rs.

STEP 8: Run `cargo check -p forge-core`. Fix errors.

STEP 9: Add tests for file_io: atomic save round-trip using temp dir, detect_line_ending for LF and CRLF.

STEP 10: Run `cargo test -p forge-core -p forge-app`.

STEP 11: Run `cargo clippy -- -D warnings` and `cargo fmt`.

DONE.
```

---

## AGENT 9 — forge-workspace + Project Detection

```
You are working on the Forge editor, a Rust workspace at the root of this repo.

Follow these steps IN ORDER.

STEP 1: Read tasks/GLOBAL_RULES.md and tasks/session1/agent_09.md.

STEP 2: Create the directory crates/forge-workspace/src/

STEP 3: Create crates/forge-workspace/Cargo.toml:
[package]
name = "forge-workspace"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = { workspace = true }
serde = { workspace = true }
toml = { workspace = true }

STEP 4: Create crates/forge-workspace/src/lib.rs with:
- Workspace struct { name: String, roots: Vec<PathBuf> } with Serialize, Deserialize
- from_dir(path: &Path) -> Result<Self>: single-root workspace from directory
- add_root(&mut self, path: PathBuf): add if not already in roots
- resolve_path(&self, relative: &str) -> Option<PathBuf>: check each root for existing file
- contains(&self, path: &Path) -> bool: check if path starts_with any root

STEP 5: Add "crates/forge-workspace" to [workspace] members in root Cargo.toml.

STEP 6: Run `cargo check -p forge-workspace`. Fix errors.

STEP 7: Add tests: single root workspace, add_root deduplication, resolve_path finds existing file.

STEP 8: Run `cargo test -p forge-workspace`. All pass.

STEP 9: Open crates/forge-core/src/project.rs. Read existing code.

STEP 10: ADD (do not replace existing code) to project.rs:
- ProjectKind enum { Rust, Node, Python, Go, Generic }
- pub fn detect_project_kind(root: &Path) -> ProjectKind: check for Cargo.toml=Rust, package.json=Node, pyproject.toml or setup.py=Python, go.mod=Go, else Generic

STEP 11: Run `cargo check -p forge-core`. Fix errors.

STEP 12: Add test for detect_project_kind: test on the repo root (should detect Rust since Cargo.toml exists).

STEP 13: Run `cargo test -p forge-workspace -p forge-core`. All pass.

STEP 14: Run `cargo clippy -- -D warnings` and `cargo fmt`.

DONE.
```

---

## AGENT 10 — Decoration Framework + Selection Rendering

```
You are working on the Forge editor, a Rust workspace at the root of this repo.

Follow these steps IN ORDER.

STEP 1: Read tasks/GLOBAL_RULES.md and tasks/session1/agent_10.md.

STEP 2: Open crates/forge-renderer/src/lib.rs. Read what modules already exist.

STEP 3: Create crates/forge-renderer/src/decorations.rs with:
- UnderlineStyle enum { Solid, Wavy, Dashed, Dotted }
- Decoration enum with three variants:
  - Underline { line: usize, start_col: usize, end_col: usize, color: [u8; 4], style: UnderlineStyle }
  - LineBackground { line: usize, color: [u8; 4] }
  - InlineText { line: usize, col: usize, text: String, color: [u8; 4] }
- DecorationLayer struct { decorations: Vec<Decoration> }
- DecorationLayer::new() -> Self
- add(&mut self, dec: Decoration)
- clear(&mut self)
- get_line_decorations(&self, line: usize) -> Vec<&Decoration>: filter by line number
- count(&self) -> usize

STEP 4: Add `pub mod decorations;` to crates/forge-renderer/src/lib.rs.

STEP 5: Run `cargo check -p forge-renderer`. Fix errors.

STEP 6: Add tests: add two decorations on different lines, get_line_decorations returns only matching, clear empties all.

STEP 7: Run `cargo test -p forge-renderer`. All pass.

STEP 8: Open crates/forge-app/src/ui.rs. Read the Zone struct (or whatever struct defines UI layout zones) — note the field names (x, y, width, height or similar).

STEP 9: Open crates/forge-app/src/rect_renderer.rs. Read the Rect struct — note its exact field names and types.

STEP 10: Create crates/forge-app/src/selection_render.rs with SelectionRenderer struct. Add render_selections() that takes: selection ranges as Vec of (start_line, start_col, end_line, end_col), scroll_top offset, and editor zone reference. Return Vec of Rect (using the EXACT Rect type from rect_renderer.rs you found in step 9). Generate translucent blue highlight rectangles for each selected range.

STEP 11: Add `mod selection_render;` to crates/forge-app/src/main.rs.

STEP 12: Run `cargo check -p forge-app`. Fix errors — make sure Rect fields match exactly.

STEP 13: Run `cargo clippy -- -D warnings` and `cargo fmt`.

DONE.
```

---

# After All 10 Complete

Merge all branches. Then for Session 2, open tasks/session2/all_agents.md — each "## Agent XX" section is one Jules session. Follow the same pattern.
