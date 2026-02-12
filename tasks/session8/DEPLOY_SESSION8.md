# SESSION 8 — LSP Integration + Live Terminal + Split Editor
# ONE JULES TASK — Copy this ENTIRE file as one Jules prompt.
# PREREQUISITE: Session 7 must be merged. `cargo check --workspace` exits 0.
# THIS SESSION MODIFIES `application.rs`.
# ═══════════════════════════════════════════════════════════════

You are working on **Forge**, a GPU-accelerated code editor written in Rust. This session wires LSP diagnostics, live terminal PTY rendering, and split editor into the render loop.

## CRITICAL CONTEXT

### Existing crates/modules you MUST use:

**`crates/forge-lsp/src/client.rs`** — LSP client that spawns language servers:
```rust
pub struct LspClient { /* internal: child process, stdin/stdout */ }
impl LspClient {
    pub async fn start(command: &str, args: &[&str]) -> Result<Self>
    pub async fn initialize(root_uri: &str) -> Result<serde_json::Value>
    pub async fn did_open(uri: &str, language: &str, text: &str) -> Result<()>
    pub async fn did_change(uri: &str, text: &str) -> Result<()>
    pub async fn completion(uri: &str, line: u32, character: u32) -> Result<Vec<CompletionItem>>
    pub async fn hover(uri: &str, line: u32, character: u32) -> Result<Option<String>>
}
```

**`crates/forge-lsp/src/protocol.rs`** — LSP protocol types:
```rust
pub struct Diagnostic { pub range: Range, pub severity: Option<u32>, pub message: String, pub source: Option<String> }
pub struct Range { pub start: Position, pub end: Position }
pub struct Position { pub line: u32, pub character: u32 }
pub struct CompletionItem { pub label: String, pub kind: Option<u32>, pub detail: Option<String>, pub insert_text: Option<String> }
```

**`crates/forge-terminal/src/lib.rs`** (from Session 3 Redo):
```rust
pub struct Terminal { pub pty: Pty, pub parser: AnsiParser, pub grid: TerminalGrid }
impl Terminal {
    pub fn new() -> Result<Self>             // Spawns shell PTY (80x24)
    pub fn resize(&mut self, cols: u16, rows: u16) -> Result<()>
    pub fn send_input(&mut self, text: &str) -> Result<()>
    pub fn tick(&mut self) -> Vec<TermEvent>  // Reads PTY, parses ANSI, updates grid. NON-BLOCKING.
    pub fn render_grid(&self) -> &TerminalGrid
}
// NOTE: There is NO poll_output() or is_alive() method. Use tick() instead.
```

**`crates/forge-app/src/split_editor.rs`** (from Session 4):
```rust
pub struct SplitLayout { root: SplitNode, active_editor: u64, next_id: u64 }
impl SplitLayout {
    pub fn new() -> Self
    pub fn split(direction: SplitDirection) -> u64
    pub fn close(editor_id: u64) -> bool
    pub fn all_editor_ids() -> Vec<u64>
}
```

### `application.rs` after Session 7:
- Uses `tab_manager: TabManager` for multi-file editing
- Sidebar renders real file tree
- Minimap, code folding, indent guides are rendered

---

## RULES (MANDATORY)

1. No `.unwrap()` in production.
2. `cargo fmt` + `cargo clippy -- -D warnings` = zero warnings.
3. LSP is async (tokio) — use `tokio::runtime::Handle` or a background thread for async calls. The render loop is synchronous (winit), so you CANNOT use `.await` directly in the event handler.
4. Terminal PTY reading must be non-blocking — poll in the render loop.
5. If any crate dependency doesn't exist or its API is different from what's documented here, **read the actual file first** and adapt.

---

## TASK 1: LSP Manager — Background Language Server

**Create `crates/forge-app/src/lsp_manager.rs`:**

Since the render loop is synchronous but LSP is async, create a manager that runs LSP on a background thread and communicates via channels.

```rust
use std::sync::mpsc;

pub enum LspRequest {
    DidOpen { uri: String, language: String, text: String },
    DidChange { uri: String, text: String },
    Completion { uri: String, line: u32, col: u32 },
    Hover { uri: String, line: u32, col: u32 },
}

pub enum LspResponse {
    Diagnostics { uri: String, diagnostics: Vec<DiagnosticInfo> },
    Completions { items: Vec<CompletionInfo> },
    Hover { text: Option<String> },
}

#[derive(Debug, Clone)]
pub struct DiagnosticInfo {
    pub line: u32,
    pub col: u32,
    pub end_line: u32,
    pub end_col: u32,
    pub severity: u32, // 1=Error, 2=Warning, 3=Info, 4=Hint
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct CompletionInfo {
    pub label: String,
    pub detail: Option<String>,
    pub insert_text: Option<String>,
}

pub struct LspManager {
    pub request_tx: Option<mpsc::Sender<LspRequest>>,
    pub response_rx: Option<mpsc::Receiver<LspResponse>>,
    pub diagnostics: Vec<DiagnosticInfo>,
    pub completions: Vec<CompletionInfo>,
    pub hover_text: Option<String>,
    pub initialized: bool,
}

impl LspManager {
    pub fn new() -> Self {
        Self {
            request_tx: None,
            response_rx: None,
            diagnostics: Vec::new(),
            completions: Vec::new(),
            hover_text: None,
            initialized: false,
        }
    }

    /// Start LSP server on background thread
    pub fn start(&mut self, language_server_cmd: &str, root_uri: &str) {
        let (req_tx, req_rx) = mpsc::channel::<LspRequest>();
        let (resp_tx, resp_rx) = mpsc::channel::<LspResponse>();
        self.request_tx = Some(req_tx);
        self.response_rx = Some(resp_rx);

        let cmd = language_server_cmd.to_string();
        let root = root_uri.to_string();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
            rt.block_on(async move {
                // Try to start LSP
                match forge_lsp::client::LspClient::start(&cmd, &[]).await {
                    Ok(mut client) => {
                        let _ = client.initialize(&root).await;
                        // Process requests
                        while let Ok(req) = req_rx.recv() {
                            match req {
                                LspRequest::DidOpen { uri, language, text } => {
                                    let _ = client.did_open(&uri, &language, &text).await;
                                }
                                LspRequest::DidChange { uri, text } => {
                                    let _ = client.did_change(&uri, &text).await;
                                }
                                LspRequest::Completion { uri, line, col } => {
                                    if let Ok(items) = client.completion(&uri, line, col).await {
                                        let infos: Vec<CompletionInfo> = items.iter().map(|i| CompletionInfo {
                                            label: i.label.clone(),
                                            detail: i.detail.clone(),
                                            insert_text: i.insert_text.clone(),
                                        }).collect();
                                        let _ = resp_tx.send(LspResponse::Completions { items: infos });
                                    }
                                }
                                LspRequest::Hover { uri, line, col } => {
                                    if let Ok(text) = client.hover(&uri, line, col).await {
                                        let _ = resp_tx.send(LspResponse::Hover { text });
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("LSP start failed: {}", e);
                    }
                }
            });
        });
        self.initialized = true;
    }

    /// Poll for responses (call in render loop — non-blocking)
    pub fn poll(&mut self) {
        if let Some(ref rx) = self.response_rx {
            while let Ok(resp) = rx.try_recv() {
                match resp {
                    LspResponse::Diagnostics { diagnostics, .. } => self.diagnostics = diagnostics,
                    LspResponse::Completions { items } => self.completions = items,
                    LspResponse::Hover { text } => self.hover_text = text,
                }
            }
        }
    }

    /// Send a request (non-blocking)
    pub fn send(&self, req: LspRequest) {
        if let Some(ref tx) = self.request_tx {
            let _ = tx.send(req);
        }
    }
}
```

Add `mod lsp_manager;` to `main.rs`.
Add `tokio = { workspace = true }` to `crates/forge-app/Cargo.toml` if not already present.

Tests: LspManager::new() compiles, poll on empty returns nothing.

## TASK 2: Wire LSP Diagnostics Rendering

**Add `lsp_manager: crate::lsp_manager::LspManager` to `AppState`.**

In `init_state()`, create: `let lsp_manager = crate::lsp_manager::LspManager::new();`

Optionally try to start rust-analyzer:
```rust
// Try to start LSP for Rust files
if self.file_path.as_ref().map(|p| p.ends_with(".rs")).unwrap_or(false) {
    lsp_manager.start("rust-analyzer", &format!("file://{}", std::env::current_dir().unwrap_or_default().display()));
}
```

**In render loop**, poll LSP and render diagnostics:
```rust
state.lsp_manager.poll();

// Render diagnostic squiggly lines under errors
for diag in &state.lsp_manager.diagnostics {
    if diag.line as usize >= scroll_top && (diag.line as usize) < scroll_top + vis_lines {
        let y = state.layout.editor.y + ((diag.line as usize - scroll_top) as f32 * LayoutConstants::LINE_HEIGHT) + LayoutConstants::LINE_HEIGHT - 2.0;
        let x = state.layout.editor.x + (diag.col as f32 * 8.5);
        let width = ((diag.end_col - diag.col).max(1) as f32) * 8.5;
        let color = match diag.severity {
            1 => [1.0, 0.3, 0.3, 0.8],  // Error: red
            2 => [1.0, 0.8, 0.2, 0.8],  // Warning: yellow
            _ => [0.3, 0.6, 1.0, 0.5],  // Info: blue
        };
        state.render_batch.push(crate::rect_renderer::RectPrimitive {
            x, y, width, height: 2.0, color, border_radius: 0.0,
        });
    }
}
```

## TASK 3: Wire Live Terminal to Bottom Panel

**Add `terminal: Option<forge_terminal::Terminal>` to `AppState`.**

Initialize: `let terminal: Option<forge_terminal::Terminal> = None;`

Add `forge-terminal = { path = "../forge-terminal" }` to `crates/forge-app/Cargo.toml`.

**When bottom panel is toggled (Ctrl+`)**, create terminal on first use:
```rust
// In the Ctrl+` handler:
if state.terminal.is_none() {
    match forge_terminal::Terminal::new() {
        Ok(term) => state.terminal = Some(term),
        Err(e) => tracing::warn!("Terminal failed: {}", e),
    }
}
```

**In render loop**, if terminal exists and bottom panel is visible, poll and render:
```rust
if let Some(ref bp_zone) = state.layout.bottom_panel {
    if self.bottom_panel.visible {
        // Poll terminal output (tick reads PTY, parses ANSI, updates grid)
        if let Some(ref mut term) = state.terminal {
            let _events = term.tick(); // Non-blocking read
        }

        // Render terminal grid as text
        if let Some(ref term) = state.terminal {
            let grid = term.render_grid();
            let mut term_text = String::new();
            for row in 0..grid.rows {
                for col in 0..grid.cols {
                    term_text.push(grid.cells[row as usize][col as usize].ch);
                }
                term_text.push('\n');
            }
            state.bottom_panel_buffer.set_text(
                &mut state.font_system,
                &term_text,
                Attrs::new().family(Family::Monospace).color(GlyphonColor::rgb(229, 229, 229)),
                Shaping::Advanced,
            );
        }
    }
}
```

**Forward keyboard input to terminal** when bottom panel is focused:
```rust
// Add a flag: bottom_panel_focused: bool to AppState
// When bottom panel is visible and user clicks in it, set focused = true
// When focused and typing, forward to terminal:
if state.bottom_panel_focused {
    if let Some(ref mut term) = state.terminal {
        match key {
            Key::Named(NamedKey::Enter) => { let _ = term.send_input("\r\n"); }
            Key::Named(NamedKey::Backspace) => { let _ = term.send_input("\x7f"); }
            Key::Character(ref c) => { let _ = term.send_input(c); }
            _ => {}
        }
        state.window.request_redraw();
        return; // Don't process as editor input
    }
}
```

## TASK 4: Problems Panel from Diagnostics

**Create `crates/forge-app/src/diagnostics_panel.rs`:**

```rust
use crate::lsp_manager::DiagnosticInfo;

pub struct DiagnosticsPanel {
    pub visible: bool,
    pub diagnostics: Vec<(String, DiagnosticInfo)>,  // (filename, diagnostic)
    pub selected: usize,
}

impl DiagnosticsPanel {
    pub fn new() -> Self { Self { visible: false, diagnostics: Vec::new(), selected: 0 } }
    pub fn toggle(&mut self) { self.visible = !self.visible; }
    pub fn update(&mut self, file: &str, diags: &[DiagnosticInfo]) {
        self.diagnostics.retain(|(f, _)| f != file);
        for d in diags {
            self.diagnostics.push((file.to_string(), d.clone()));
        }
        self.diagnostics.sort_by(|a, b| a.1.severity.cmp(&b.1.severity));
    }
    pub fn error_count(&self) -> usize { self.diagnostics.iter().filter(|(_, d)| d.severity == 1).count() }
    pub fn warning_count(&self) -> usize { self.diagnostics.iter().filter(|(_, d)| d.severity == 2).count() }
}
```

Add `mod diagnostics_panel;` to `main.rs`.

**Update status bar** to show error/warning counts:
```rust
let errors = state.diagnostics_panel.error_count();
let warnings = state.diagnostics_panel.warning_count();
let status_text = format!(
    "  Forge IDE  │  Ln {}, Col {}  │  UTF-8  │  {}  │  ⚠ {}  ✕ {}  │  {} fps",
    cursor_line, cursor_col, language, warnings, errors, fps
);
```

## TASK 5: Split Editor Layout

**Add `split_layout: crate::split_editor::SplitLayout` to `AppState`.**

Read `split_editor.rs` to verify API. Initialize: `let split_layout = crate::split_editor::SplitLayout::new();`

**Wire Ctrl+\ shortcut** to split editor:
```rust
Key::Character(ref c) if c == "\\" && ctrl => {
    state.split_layout.split(crate::split_editor::SplitDirection::Vertical);
    state.window.request_redraw();
}
```

Note: Full split rendering (dividing the editor zone into multiple sub-zones and rendering an Editor per zone) is complex. For this session, just add the data structures and the shortcut. The actual multi-pane rendering where each split has its own editor buffer is a stretch goal — implement it if you have time, otherwise leave a TODO comment explaining what remains.

## TASK 6: Bracket Matching Highlighting

**Wire `crate::bracket_match` into the render loop:**

Read `bracket_match.rs` to find the API. It likely has a function like `find_matching_bracket(text: &str, cursor_pos: usize) -> Option<usize>`.

In the render loop, if cursor is on a bracket, highlight both brackets:
```rust
if let Some(editor) = state.tab_manager.active_editor() {
    // Get cursor byte offset
    let text = editor.buffer.text();
    let line = editor.cursor_line();
    let col = editor.cursor_col();
    // Convert to byte position
    let line_start = editor.buffer.rope().line_to_byte(line);
    let cursor_byte = line_start + col;

    if let Some(match_pos) = crate::bracket_match::find_matching_bracket(&text, cursor_byte) {
        // Highlight match position
        let match_line = editor.buffer.rope().byte_to_line(match_pos);
        let match_col = match_pos - editor.buffer.rope().line_to_byte(match_line);
        if match_line >= scroll_top && match_line < scroll_top + vis_lines {
            let x = state.layout.editor.x + (match_col as f32 * 8.5);
            let y = state.layout.editor.y + ((match_line - scroll_top) as f32 * LayoutConstants::LINE_HEIGHT);
            state.render_batch.push(crate::rect_renderer::RectPrimitive {
                x, y,
                width: 8.5, height: LayoutConstants::LINE_HEIGHT,
                color: [1.0, 1.0, 1.0, 0.08],
                border_radius: 0.0,
            });
        }
    }
}
```

**Read `bracket_match.rs` FIRST** to verify the actual function name and signature.

---

## FINAL VERIFICATION

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test --workspace
cargo check --workspace
```

ALL FOUR must exit 0. If `forge-terminal` or `forge-lsp` APIs differ from documented, adapt code to match reality. Read files FIRST, code SECOND.
