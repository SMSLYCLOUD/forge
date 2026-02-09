# SESSION 1 — Foundation & Cleanup
# ONE JULES TASK — Copy this ENTIRE file as one Jules prompt.
# This is a SINGLE task that creates/modifies multiple files.
# ═══════════════════════════════════════════════════════════════

You are working on **Forge**, a GPU-accelerated code editor written in Rust. You will complete ALL tasks listed below in sequence within this single session. Each task creates new files. Do them ALL.

---

## RULES (READ FIRST — MANDATORY)

1. **Rust 2021 edition.** No `.unwrap()` in production — use `Result<T, E>` + `thiserror`. `.unwrap()` only in `#[cfg(test)]` blocks.
2. **Every public function** MUST have ≥1 unit test in a `#[cfg(test)] mod tests` block.
3. Run `cargo fmt` + `cargo clippy -- -D warnings` after completing ALL tasks. Zero warnings.
4. Only add dependencies explicitly listed below. Add to `[workspace.dependencies]` in ROOT `Cargo.toml`, reference as `{ workspace = true }` in crate `Cargo.toml`.
5. **Do NOT modify** `crates/forge-app/src/application.rs`. That file is only modified in Session 5.
6. UTF-8, LF line endings. `///` for public API doc comments, `//` for internal.
7. When creating a new crate, add its path to `[workspace] members` in ROOT `Cargo.toml`.

---

## WORKSPACE STRUCTURE (CURRENT STATE)

```
forge/
├── Cargo.toml              ← Workspace root. YOU WILL MODIFY THIS.
├── crates/
│   ├── forge-core/         ← Rope buffer, transactions, undo/redo
│   ├── forge-renderer/     ← wgpu GPU rendering
│   ├── forge-window/       ← winit windowing
│   ├── forge-app/          ← Main app (DO NOT MODIFY application.rs)
│   ├── forge-config/       ← TOML config (EXISTS — you will rewrite contents)
│   ├── forge-theme/        ← Color themes (EXISTS — you will rewrite contents)
│   ├── forge-input/        ← Keyboard/mouse input (EXISTS — you will add files)
│   ├── forge-surfaces/     ← UI surfaces
│   ├── forge-confidence/   ← Confidence engine
│   ├── forge-propagation/  ← Change propagation
│   ├── forge-semantic/     ← Semantic analysis
│   ├── forge-bayesnet/     ← Bayesian network
│   ├── forge-ml/           ← ML inference
│   ├── forge-anticipation/ ← Anticipation engine
│   ├── forge-immune/       ← Anomaly detection
│   ├── forge-developer/    ← Developer profiles
│   ├── forge-feedback/     ← Feedback tracking
│   ├── forge-agent/        ← AI agent
│   └── forge-net/          ← Network client
```

---

## CURRENT ROOT Cargo.toml `[workspace]` MEMBERS

```toml
[workspace]
members = [
    "crates/forge-core",
    "crates/forge-renderer",
    "crates/forge-window",
    "crates/forge-app",
    "crates/forge-config",
    "crates/forge-theme",
    "crates/forge-input",
    "crates/forge-confidence",
    "crates/forge-propagation",
    "crates/forge-semantic",
    "crates/forge-bayesnet",
    "crates/forge-ml",
    "crates/forge-anticipation",
    "crates/forge-immune",
    "crates/forge-developer",
    "crates/forge-surfaces",
    "crates/forge-feedback",
    "crates/forge-agent",
    "crates/forge-net",
]
resolver = "2"
```

**You MUST add these new crates to the members list:**
- `"crates/forge-types"`
- `"crates/forge-keybindings"`
- `"crates/forge-workspace"`

---

## CURRENT WORKSPACE DEPENDENCIES

```toml
[workspace.dependencies]
ropey = "1"
smallvec = "1"
unicode-segmentation = "1"
encoding_rs = "0.8"
wgpu = "23"
cosmic-text = "0.12"
glyphon = "0.7"
winit = "0.30"
pollster = "0.4"
arboard = "3"
toml = "0.8"
serde = { version = "1", features = ["derive"] }
anyhow = "1"
thiserror = "2"
tracing = "0.1"
tracing-subscriber = "0.3"
proptest = "1"
rusqlite = { version = "0.31", features = ["bundled"] }
petgraph = "0.6"
uuid = { version = "1", features = ["v4", "serde"] }
serde_json = "1"
git2 = "0.20"
reqwest = { version = "0.12", features = ["json", "stream", "rustls-tls", "gzip", "brotli"] }
tokio = { version = "1", features = ["full"] }
futures = "0.3"
async-trait = "0.1"
dirs-next = "2"
rand = "0.8"
bytemuck = { version = "1", features = ["derive"] }
```

**You MUST add this to workspace deps:**
- `forge-types = { path = "crates/forge-types" }`

---

## TASK 1: Fix All Compiler Warnings

1. Run `cargo check --workspace 2>&1` and read ALL warnings.
2. For EVERY warning, fix it:
   - Dead code: Remove if truly dead. If placeholder, add `#[allow(dead_code)] // TODO: future use`
   - Unused imports: Remove them.
   - Unused variables: Prefix with `_`
   - Missing docs: Add `///` doc comments
3. Result: `cargo check --workspace` produces ZERO warnings.

---

## TASK 2: Create `forge-types` Crate

Shared types used across all Forge crates.

**Create `crates/forge-types/Cargo.toml`:**
```toml
[package]
name = "forge-types"
version.workspace = true
edition.workspace = true

[dependencies]
serde = { workspace = true }

[dev-dependencies]
proptest = { workspace = true }
```

**Create `crates/forge-types/src/lib.rs`** with these types and full doc comments:

- `Color` struct: `r: f32, g: f32, b: f32, a: f32` with `#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]`
  - `const fn rgba(r, g, b, a) -> Self`
  - `const fn rgb(r, g, b) -> Self` (a = 1.0)
  - `fn to_u8_array(&self) -> [u8; 4]`
- `Rect` struct: `x: f32, y: f32, width: f32, height: f32`
  - `const fn new(x, y, width, height) -> Self`
  - `fn contains(&self, px: f32, py: f32) -> bool`
- `Position` struct: `line: usize, col: usize` with `Eq, Hash`
- `Size` struct: `width: f32, height: f32`

**Tests (≥4):** color_to_u8, color_alpha, rect_contains, position_equality.

Add `"crates/forge-types"` to workspace members and `forge-types = { path = "crates/forge-types" }` to workspace deps.

---

## TASK 3: Rewrite `forge-config` (Full TOML Configuration)

The crate exists at `crates/forge-config/`. Rewrite its contents.

**Rewrite `crates/forge-config/src/lib.rs`** with:

- `ForgeConfig` struct with `#[serde(default)]`:
  - `editor: EditorConfig`
  - `theme: ThemeConfig`
  - `keybindings: KeybindingConfig`
  - `terminal: TerminalConfig`
  - `git: GitConfig`

- `EditorConfig` (all with `#[serde(default)]`):
  - `font_size: f32` (default 14.0)
  - `font_family: String` (default "JetBrains Mono")
  - `tab_size: u32` (default 4)
  - `insert_spaces: bool` (default true)
  - `line_numbers: bool` (default true)
  - `highlight_current_line: bool` (default true)
  - `word_wrap: bool` (default false)
  - `minimap: bool` (default true)
  - `bracket_matching: bool` (default true)
  - `auto_indent: bool` (default true)
  - `format_on_save: bool` (default false)
  - `cursor_blink_ms: u32` (default 500)

- `ThemeConfig`: `name: String` (default "Forge Dark"), `dark_mode: bool` (default true)
- `TerminalConfig`: `shell: Option<String>`, `font_size: f32` (13.0), `scrollback: u32` (10_000)
- `GitConfig`: `enabled: bool` (true), `decorations: bool` (true)

- `ConfigError` enum using `thiserror` for IO, TOML parse, and serialize errors.

- `ForgeConfig::load() -> Result<Self, ConfigError>` — load from `~/.config/forge/config.toml`, return defaults if missing
- `ForgeConfig::load_from(path) -> Result<Self, ConfigError>`
- `ForgeConfig::save(&self) -> Result<(), ConfigError>` — save to default path, create dirs
- `ForgeConfig::config_path() -> PathBuf` — uses `dirs_next::config_dir()`

**Update `crates/forge-config/Cargo.toml`** deps: `serde`, `toml`, `thiserror`, `dirs-next` (all `{ workspace = true }`).

**Tests (≥5):** default values correct, TOML round-trip, partial TOML fills defaults, load_from temp file, empty TOML = defaults.

---

## TASK 4: Rewrite `forge-theme` (Complete Theme Engine)

The crate exists at `crates/forge-theme/`. Rewrite its contents.

**Rewrite `crates/forge-theme/src/lib.rs`** with:

- `Theme` struct with 30+ named color fields (define a local `Color` struct or reuse from forge-types):
  - Background/foreground: `background`, `foreground`, `cursor`, `selection`, `current_line`
  - Line numbers: `line_number`, `line_number_active`
  - Chrome: `sidebar_bg`, `sidebar_fg`, `statusbar_bg`, `statusbar_fg`
  - Tabs: `tab_active_bg`, `tab_inactive_bg`, `tab_active_fg`, `tab_inactive_fg`
  - Diagnostics: `error`, `warning`, `info`, `hint`
  - Git: `diff_added`, `diff_removed`, `diff_modified`
  - UI: `popup_bg`, `popup_fg`, `bracket_match`, `find_match`

- `TokenColors` struct for syntax: `keyword`, `function`, `type_name`, `string`, `number`, `comment`, `operator`, `punctuation`, `variable`, `constant`

- 7 built-in themes as `impl Theme` static functions:
  - `forge_dark()` — #1e1e2e background, #cdd6f4 foreground
  - `forge_light()` — #f5f5f5 background, #1e1e1e foreground
  - `monokai()` — classic Monokai
  - `dracula()` — #282a36 bg, #f8f8f2 fg
  - `one_dark()` — One Dark Pro
  - `solarized_dark()` — Solarized Dark
  - `nord()` — Nord palette

- `ThemeManager` struct: `current() -> &Theme`, `set_theme(name) -> Result<()>`, `list_themes() -> Vec<&str>`

**Update `crates/forge-theme/Cargo.toml`** deps: `serde`, `serde_json`, `thiserror`.

**Tests (≥7):** each theme returns valid colors (0.0-1.0), list_themes = 7, set_theme works, set_theme("bad") errors, default = "Forge Dark", token colors are distinct.

---

## TASK 5: Create `forge-keybindings` Crate

Brand new crate.

**Create `crates/forge-keybindings/Cargo.toml`** with deps: `serde`, `serde_json`, `thiserror`, `anyhow`.

**Create `crates/forge-keybindings/src/lib.rs`** with:

- `KeyModifiers` struct: `ctrl, shift, alt, meta` (all `bool`)
- `KeyCombo` struct: `key: String`, `modifiers: KeyModifiers`
- `Keybinding` struct: `combo: Vec<KeyCombo>` (chord support), `command: String`, `when: Option<String>`
- `KeybindingResolver` struct:
  - `new() -> Self` — registers 30+ default keybindings
  - `add(binding)`, `resolve(combo) -> Option<&str>`, `resolve_chord(combos) -> Option<&str>`, `find_binding(command) -> Option<&Keybinding>`

- Default keybindings (at minimum):
  Ctrl+S=file.save, Ctrl+O=file.open, Ctrl+N=file.new, Ctrl+W=tab.close, Ctrl+Z=edit.undo, Ctrl+Y=edit.redo, Ctrl+X=edit.cut, Ctrl+C=edit.copy, Ctrl+V=edit.paste, Ctrl+A=edit.select_all, Ctrl+F=find.open, Ctrl+H=find.replace, Ctrl+G=editor.go_to_line, Ctrl+P=file_picker.open, Ctrl+Shift+P=command_palette.open, Ctrl+`=terminal.toggle, Ctrl+/=editor.toggle_comment, F2=editor.rename_symbol, F12=editor.go_to_definition, Ctrl+Shift+F=search.open, Ctrl+,=settings.open, Ctrl+D=editor.select_next_occurrence, Ctrl+Shift+L=editor.select_all_occurrences, Ctrl+Tab=tab.next, Ctrl+Shift+Tab=tab.previous, Shift+F12=editor.find_references, Ctrl+Shift+O=editor.go_to_symbol, Ctrl+Shift+S=file.save_as, Ctrl+\=editor.split_right, Alt+Z=editor.toggle_word_wrap

Add `"crates/forge-keybindings"` to workspace members.

**Tests (≥6):** resolve Ctrl+S, resolve unbound=None, find_binding, chord detection, custom override, conflict override.

---

## TASK 6: Add Clipboard to forge-input

**Create `crates/forge-input/src/clipboard.rs`:**

- `ClipboardError` enum with `thiserror`: `Access(String)`, `Empty`
- `Clipboard` struct wrapping `Option<arboard::Clipboard>`
  - `new() -> Self` — create clipboard (Ok or None if headless)
  - `copy(&mut self, text: &str) -> Result<(), ClipboardError>`
  - `paste(&mut self) -> Result<String, ClipboardError>`
  - `cut(&mut self, text: &str) -> Result<String, ClipboardError>` — copy + return text
  - `is_available(&self) -> bool`

**Add `pub mod clipboard;` to `crates/forge-input/src/lib.rs` (or equivalent entry file).**
**Ensure `arboard = { workspace = true }`** is in `crates/forge-input/Cargo.toml`.

**Tests (≥2):** new() doesn't panic, is_available returns bool. Mark clipboard round-trip test as `#[ignore]`.

---

## TASK 7: Create `forge-workspace` Crate

Brand new crate for multi-root workspace support.

**Create `crates/forge-workspace/Cargo.toml`** with deps: `serde`, `serde_json`, `anyhow`, `thiserror`.

**Create `crates/forge-workspace/src/lib.rs`** with:

- `Workspace` struct: `roots: Vec<WorkspaceRoot>`, `name: Option<String>`
- `WorkspaceRoot` struct: `path: PathBuf`, `name: String`
- `Workspace::open(path) -> Result<Self>` — single-root
- `Workspace::open_multi(paths) -> Result<Self>` — multi-root
- `Workspace::load_from_file(path) -> Result<Self>` — load `.forge-workspace` JSON
- `Workspace::save_to_file(path) -> Result<()>`
- `Workspace::resolve_path(relative) -> Option<PathBuf>`
- `Workspace::find_file(name) -> Vec<PathBuf>` — search across roots

Add `"crates/forge-workspace"` to workspace members.

**Tests (≥5):** open single root, open multi root, resolve_path, save/load round-trip, find_file.

---

## TASK 8: Project Detection in forge-core

**Create `crates/forge-core/src/project.rs`:**

- `ProjectKind` enum: `Rust`, `Node`, `Python`, `Go`, `Generic`
- `ProjectInfo` struct: `root: PathBuf`, `kind: ProjectKind`, `name: String`
- `detect_project(path) -> Option<ProjectInfo>` — scan for Cargo.toml/package.json/pyproject.toml/go.mod/.git, walk up parents
- `scan_directory(path) -> Vec<PathBuf>` — recursive file list, skip `node_modules`, `target`, `__pycache__`, `.git`, hidden dirs
- `is_binary_file(path) -> bool` — read first 512 bytes, check for null bytes

**Add `pub mod project;` to `crates/forge-core/src/lib.rs`.**

**Tests (≥5):** detect Rust project, detect Node project, None for empty dir, scan_directory skips node_modules, is_binary_file.

---

## TASK 9: Error Recovery in forge-core

**Create `crates/forge-core/src/recovery.rs`:**

- `RecoveryManager` struct: `recovery_dir: PathBuf`
- `RecoveryEntry` struct: `file_path: PathBuf`, `recovery_path: PathBuf`, `timestamp: SystemTime`
- `RecoveryManager::new() -> Result<Self>` — create recovery dir at `~/.config/forge/recovery/`
- `save_state(file_path, content) -> Result<()>` — write to recovery dir (hash path for filename)
- `restore_state(file_path) -> Result<Option<String>>` — read recovery if exists
- `has_recovery(file_path) -> bool`
- `clear_recovery(file_path) -> Result<()>` — delete recovery file
- `list_recoveries() -> Result<Vec<RecoveryEntry>>`
- `clean_old_recoveries(max_age: Duration) -> Result<usize>`

**Add `pub mod recovery;` to `crates/forge-core/src/lib.rs`.**

**Tests (≥5):** save/restore round-trip, clear_recovery, has_recovery bool, list_recoveries, clean.

---

## TASK 10: File I/O Improvements in forge-core

**Create `crates/forge-core/src/file_io.rs`:**

- `FileEncoding` enum: `Utf8`, `Utf16Le`, `Utf16Be`, `Latin1`
- `LineEnding` enum: `Lf`, `Crlf`, `Mixed`
- `FileInfo` struct: `encoding`, `line_ending`, `is_readonly: bool`, `is_binary: bool`, `size_bytes: u64`
- `detect_encoding(bytes) -> FileEncoding` — check BOM, try UTF-8, fallback Latin1
- `detect_line_ending(text) -> LineEnding` — count \r\n vs \n
- `read_file(path) -> Result<(String, FileInfo)>` — read with encoding detection
- `write_file_atomic(path, content, line_ending) -> Result<()>` — write to temp then rename
- `is_binary_file(path) -> Result<bool>` — check for null bytes in first 8KB

**Ensure `encoding_rs = { workspace = true }` in `crates/forge-core/Cargo.toml`.**
**Add `pub mod file_io;` to `crates/forge-core/src/lib.rs`.**

**Tests (≥6):** detect UTF-8 BOM, detect UTF-16 LE BOM, detect plain UTF-8, detect LF vs CRLF, atomic write+read round-trip, is_binary_file.

---

## FINAL VERIFICATION (RUN ALL FOUR)

After completing ALL 10 tasks above:

```bash
cargo fmt --check        # Must exit 0
cargo clippy -- -D warnings  # Must exit 0
cargo test --workspace   # Must exit 0
cargo check --workspace  # Must exit 0
```

**ALL FOUR must pass.** Fix any failures before submitting.

## SUMMARY OF ALL FILES CREATED/MODIFIED

| Action | File |
|--------|------|
| MODIFY | `Cargo.toml` (root) — add 3 new crates to members, add forge-types to deps |
| MODIFY | `crates/forge-config/Cargo.toml` — update deps |
| MODIFY | `crates/forge-config/src/lib.rs` — full rewrite |
| MODIFY | `crates/forge-theme/Cargo.toml` — update deps |
| MODIFY | `crates/forge-theme/src/lib.rs` — full rewrite |
| MODIFY | `crates/forge-input/Cargo.toml` — add arboard dep |
| MODIFY | `crates/forge-input/src/lib.rs` — add `pub mod clipboard;` |
| MODIFY | `crates/forge-core/Cargo.toml` — ensure encoding_rs dep |
| MODIFY | `crates/forge-core/src/lib.rs` — add `pub mod project; pub mod recovery; pub mod file_io;` |
| CREATE | `crates/forge-types/Cargo.toml` |
| CREATE | `crates/forge-types/src/lib.rs` |
| CREATE | `crates/forge-keybindings/Cargo.toml` |
| CREATE | `crates/forge-keybindings/src/lib.rs` |
| CREATE | `crates/forge-workspace/Cargo.toml` |
| CREATE | `crates/forge-workspace/src/lib.rs` |
| CREATE | `crates/forge-input/src/clipboard.rs` |
| CREATE | `crates/forge-core/src/project.rs` |
| CREATE | `crates/forge-core/src/recovery.rs` |
| CREATE | `crates/forge-core/src/file_io.rs` |
| MODIFY | Various crates — fix compiler warnings (Task 1) |

**DO NOT modify `crates/forge-app/src/application.rs`.**
