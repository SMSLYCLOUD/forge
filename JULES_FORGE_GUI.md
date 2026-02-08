# 🔥 FORGE IDE — Complete VS Code UI + Living Intelligence Layer

> **One-shot autonomous ticket.** Build Forge into a production-grade GPU-accelerated code editor
> with VS Code layout, sub-binary intelligence, anti-crash guardrails, and turbo performance.

## Current State

The foundation is built and compiling:

| Component | Status | Files |
|-----------|--------|-------|
| GPU init (wgpu 23) | ✅ Done | `forge-app/src/gpu.rs` |
| Window (winit 0.30) | ✅ Done | `forge-app/src/main.rs` |
| Text rendering (glyphon 0.7) | ✅ Done | `forge-app/src/application.rs` |
| Editor state (rope + transactions) | ✅ Done | `forge-app/src/editor.rs` |
| Buffer API (undo/redo/selections) | ✅ Done | `forge-core/src/buffer.rs` |
| Confidence engine | ✅ Done | `forge-confidence/` |
| Intelligence crates | ✅ Stubs | `forge-bayesnet/`, `forge-propagation/`, `forge-semantic/`, `forge-immune/`, `forge-anticipation/`, `forge-surfaces/` |
| VS Code UI chrome | ❌ Missing | needs `rect_renderer.rs`, `ui.rs` |
| Syntax highlighting | ❌ Missing | needs tree-sitter integration |
| Status bar / tabs / sidebar | ❌ Missing | needs full UI layout |
| Crash guardrails | ❌ Missing | needs panic handler, recovery |
| Performance layer | ❌ Missing | needs frame budget, lazy rendering |
| AI Agent | ❌ Missing | needs `forge-agent/`, LLM integration, chat panel |

**Tech stack:** Rust, wgpu 23, winit 0.30, glyphon 0.7 (cosmic-text), ropey

---

## TASK 1: Rectangle Renderer (GPU Quad Pipeline)

**Create `forge-app/src/rect_renderer.rs`**

Build a minimal wgpu pipeline that renders solid-colored rectangles. This is the foundation for ALL UI chrome (activity bar, tabs, status bar, sidebar backgrounds, selection highlights, cursor block).

### Implementation

```rust
// Inline WGSL shader for colored quads
// Vertex: position (vec2<f32>) + color (vec4<f32>)
// Converts pixel coords → NDC in vertex shader
// Renders as triangle-list (6 verts per quad)

pub struct RectRenderer {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    vertices: Vec<RectVertex>,
    uniform_buffer: wgpu::Buffer,  // screen dimensions
    bind_group: wgpu::BindGroup,
}

impl RectRenderer {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self;
    pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: [f32; 4]);
    pub fn render<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>);
    pub fn upload(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, width: u32, height: u32);
    pub fn clear(&mut self);
}
```

### WGSL Shader

```wgsl
struct Uniforms { screen_size: vec2<f32> }
@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let ndc_x = (in.position.x / uniforms.screen_size.x) * 2.0 - 1.0;
    let ndc_y = 1.0 - (in.position.y / uniforms.screen_size.y) * 2.0;
    out.clip_pos = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
    out.color = in.color;
    return out;
}

@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
```

### Acceptance Criteria
- [ ] Renders solid-colored rectangles at pixel coordinates
- [ ] Batches all rects into one draw call per frame
- [ ] Does NOT crash on empty rect list
- [ ] Does NOT crash on window resize to 0x0

---

## TASK 2: VS Code UI Layout System

**Create `forge-app/src/ui.rs`**

Define the VS Code layout zones as a pure data structure that recomputes on window resize.

### Layout Map (pixel coordinates)

```
+------+-------------------------------------------+
| 48px |  Tab Bar (35px)                            |
|      +-------------------------------------------+
| ACT  |  Breadcrumbs (22px)                       |
| BAR  +-------------------------------------------+
|      |           Editor Area                      |
| ICONS|  Gutter  |  Code Content                   |
|      |  (60px)  |                                 |
|      |          |                                  |
+------+-------------------------------------------+
|  Status Bar (22px)                                |
+--------------------------------------------------+
```

### Color Scheme (VS Code Dark+)

```rust
pub mod colors {
    // Activity bar
    pub const ACTIVITY_BAR_BG: [f32; 4] = [0.20, 0.20, 0.20, 1.0];      // #333333
    pub const ACTIVITY_BAR_FG: [f32; 4] = [1.0, 1.0, 1.0, 0.6];         // white 60%

    // Title / tab bar
    pub const TAB_BAR_BG: [f32; 4] = [0.15, 0.15, 0.15, 1.0];          // #252526
    pub const TAB_ACTIVE_BG: [f32; 4] = [0.12, 0.12, 0.12, 1.0];       // #1e1e1e
    pub const TAB_INACTIVE_BG: [f32; 4] = [0.17, 0.17, 0.18, 1.0];     // #2d2d2d

    // Breadcrumb bar
    pub const BREADCRUMB_BG: [f32; 4] = [0.12, 0.12, 0.12, 1.0];       // #1e1e1e

    // Editor
    pub const EDITOR_BG: [f32; 4] = [0.12, 0.12, 0.12, 1.0];           // #1e1e1e
    pub const GUTTER_BG: [f32; 4] = [0.12, 0.12, 0.12, 1.0];           // same as editor
    pub const LINE_NUMBER_FG: [f32; 4] = [0.53, 0.53, 0.53, 1.0];      // #858585
    pub const CURRENT_LINE_BG: [f32; 4] = [1.0, 1.0, 1.0, 0.04];       // subtle highlight
    pub const CURSOR_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 0.8];           // white cursor
    pub const SELECTION_BG: [f32; 4] = [0.17, 0.34, 0.56, 0.5];        // #264f89 50%

    // Status bar
    pub const STATUS_BAR_BG: [f32; 4] = [0.0, 0.48, 0.80, 1.0];        // #007acc
    pub const STATUS_BAR_FG: [f32; 4] = [1.0, 1.0, 1.0, 1.0];          // white

    // AI chat panel
    pub const AI_PANEL_BG: [f32; 4] = [0.10, 0.10, 0.10, 1.0];         // #1a1a1a
    pub const AI_USER_MSG_BG: [f32; 4] = [0.0, 0.48, 0.80, 0.15];      // blue tint
    pub const AI_AGENT_MSG_BG: [f32; 4] = [0.14, 0.14, 0.14, 1.0];     // #242424
    pub const AI_INPUT_BG: [f32; 4] = [0.18, 0.18, 0.18, 1.0];         // #2e2e2e

    // Text
    pub const TEXT_FG: [f32; 4] = [0.84, 0.86, 0.88, 1.0];             // #d4d4d4
    pub const KEYWORD_FG: [f32; 4] = [0.34, 0.61, 0.77, 1.0];          // #569cd6
    pub const STRING_FG: [f32; 4] = [0.81, 0.54, 0.45, 1.0];           // #ce9178
    pub const COMMENT_FG: [f32; 4] = [0.42, 0.56, 0.31, 1.0];          // #6a9955
    pub const FUNCTION_FG: [f32; 4] = [0.86, 0.86, 0.67, 1.0];         // #dcdcaa
    pub const TYPE_FG: [f32; 4] = [0.31, 0.78, 0.76, 1.0];             // #4ec9b0
    pub const NUMBER_FG: [f32; 4] = [0.71, 0.81, 0.65, 1.0];           // #b5cea8
}

pub struct Layout {
    pub width: f32,
    pub height: f32,
    // Zones (x, y, w, h)
    pub activity_bar: Rect,
    pub tab_bar: Rect,
    pub breadcrumb_bar: Rect,
    pub gutter: Rect,
    pub editor: Rect,
    pub status_bar: Rect,
    pub ai_panel: Option<Rect>,  // toggleable right panel
}

impl Layout {
    pub fn compute(width: f32, height: f32, ai_panel_open: bool) -> Self;
}
```

### Acceptance Criteria
- [ ] All zones tile perfectly with zero gaps and zero overlap at any window size
- [ ] Minimum window size clamped to 400×300 — no layout collapse
- [ ] Layout recomputes instantly on resize, no allocation
- [ ] AI panel properly splits editor when open

---

## TASK 3: Tab Bar

**Render in `application.rs` render function**

### Features
- Active tab: brighter background, white text, colored bottom border (2px, #007acc)
- Inactive tabs: dimmer background, gray text
- Tab text: filename only (not full path)
- Tab close button: `×` that appears on hover (can start as always-visible)
- `+` new tab button at end of tab row
- Tab bar background fills remaining space after tabs

### Rendering
- Use `RectRenderer` for tab backgrounds and borders
- Use `glyphon` TextArea for tab labels, positioned per-tab

---

## TASK 4: Activity Bar (Left Icon Strip)

**48px wide, dark vertical bar on far left**

### Icons (render as Unicode text for now, replace with real icons later)

| Icon | Label | Unicode |
|------|-------|---------|
| Files | Explorer | `📁` or `☰` |
| Search | Search | `🔍` |
| Git | Source Control | `⎇` or `Y` |
| Debug | Run & Debug | `▶` |
| Extensions | Extensions | `⊞` |
| AI Agent | AI Chat | `🤖` |
| Settings (bottom) | Settings | `⚙` |

### Behavior
- Each icon is 48×48 centered in the bar
- Active icon: white, left border (2px #007acc)
- Inactive icon: gray (60% opacity)
- Click toggles sidebar (sidebar can start as hidden)
- AI icon toggles the AI chat panel (TASK 13)

---

## TASK 5: Editor Gutter (Line Numbers)

**Separate gutter zone, 60px wide**

### Features
- Right-aligned line numbers in gutter
- Current line number: white, bold
- Other line numbers: gray (#858585)
- Current line: subtle background highlight across full editor width
- Gutter separated from code by 1px border (#333)

---

## TASK 6: Status Bar

**22px tall bar at bottom, VS Code blue (#007acc)**

### Content (left to right)
- **Left side:** `⎇ master` (git branch) | `0 errors, 0 warnings`
- **Right side:** `Ln {line}, Col {col}` | `Spaces: 4` | `UTF-8` | `Rust` | `🤖 AI: Ready` | `🔥 Forge`

### Implementation
- Use `RectRenderer` for blue background
- Use `glyphon` TextArea for status text
- Update on every cursor move and file change
- Show AI agent connection status

---

## TASK 7: Cursor Rendering

**Block cursor and I-beam cursor**

### Implementation
- Render cursor as a 2px wide | bar (I-beam style like VS Code) using `RectRenderer`
- Cursor position derived from editor's `cursor_line_col()`
- Convert (line, col) to pixel (x, y) using font metrics
- Cursor blinks: 500ms on, 500ms off (use `Instant` timer, request redraw on tick)
- Cursor color: white 80% opacity

### Selection Rendering
- When selection active, render highlighted background rectangles per line
- Selection color: `#264f89` at 50% opacity

---

## TASK 8: Breadcrumb Bar

**22px tall bar below tabs**

### Content
- File path as breadcrumb: `forge-app > src > main.rs`
- Separator: ` › ` (chevron)
- Text color: gray, clickable items slightly brighter

---

## TASK 9: Anti-Crash Guardrails 🛡️

**The editor must NEVER crash. Period.**

### Panic Handler (`forge-app/src/main.rs`)

```rust
use std::panic;

fn main() {
    panic::set_hook(Box::new(|info| {
        // Log to file, never lose user data
        let crash_log = format!(
            "FORGE CRASH at {}: {}\n{}",
            chrono::Utc::now(),
            info,
            std::backtrace::Backtrace::capture()
        );
        let _ = std::fs::write("forge_crash.log", &crash_log);
        eprintln!("Forge encountered an error. Crash log saved to forge_crash.log");
    }));

    // ... existing main
}
```

### Guardrails to Implement

| Guardrail | Location | Rule |
|-----------|----------|------|
| Surface lost recovery | `gpu.rs` | On `SurfaceError::Lost`, reconfigure. Never panic. |
| Adapter not found | `gpu.rs` | Try software fallback before erroring. |
| File open failure | `editor.rs` | Show error in status bar, don't crash. Open empty buffer. |
| Save failure | `editor.rs` | Show error in status bar, don't crash. Keep buffer dirty. |
| OOM on large file | `editor.rs` | Cap file size at 100MB. Show warning for files >10MB. |
| Malformed UTF-8 | `editor.rs` | Use lossy conversion, show warning. |
| Shader compilation | `rect_renderer.rs` | Validate at startup. Panic early with clear message if GPU doesn't support required features. |
| Zero-size window | `application.rs` | Skip render when width or height is 0. |
| Glyphon prepare fail | `application.rs` | Log and skip frame, don't crash. |
| Font system init | `application.rs` | Fallback to built-in font if system fonts unavailable. |
| Index out of bounds | `editor.rs` | All cursor/scroll operations use `.min()` and `.max()` clamping. |
| Integer overflow | `ui.rs` | Use `saturating_sub`, `saturating_add` everywhere. |
| AI request timeout | `forge-agent` | 30s timeout on all LLM calls. Show "timeout" in chat, don't hang. |
| AI response parsing | `forge-agent` | Gracefully handle malformed JSON/streaming. Never crash on bad LLM output. |

### Recovery Strategy
- On ANY render error: skip that frame, request redraw next frame
- On file error: show error text in status bar for 3 seconds
- On GPU error: attempt surface reconfiguration up to 3 times
- On AI error: show error in chat panel, agent stays ready for next request

---

## TASK 10: Turbo Performance ⚡

**Target: 144 FPS with zero frame drops on files up to 100K lines**

### Frame Budget System

```rust
pub struct FrameBudget {
    target_fps: u32,              // 144
    frame_budget_us: u64,         // 6944 microseconds
    last_frame_time: Instant,
    frame_times: VecDeque<u64>,   // rolling 60-frame window
}
```

### Performance Rules

| Rule | Implementation |
|------|---------------|
| Lazy rendering | Only redraw on input events, NOT every frame. Use `ControlFlow::Wait`. |
| Visible-only rendering | Only shape/render lines visible in viewport, not entire buffer. |
| Batch draw calls | All rects in ONE draw call. All text in ONE prepare+render. |
| Pre-allocated buffers | Rect vertex buffer sized for 256 rects. Grow only if needed. |
| Dirty tracking | Track if text changed since last render. Skip re-shaping if not. |
| Scroll optimization | On scroll, only reshape visible lines, don't rebuild entire display string. |
| Font glyph caching | glyphon handles this, but ensure atlas doesn't thrash (reuse across frames). |
| Avoid allocations in render | Pre-allocate display String. Use `clear()` + `push_str()`, not `String::new()`. |
| Profile gate | Add `#[cfg(feature = "profile")]` timing to render function. |
| Frame time display | Show in status bar: `⚡ 0.8ms` (render time in milliseconds). |
| AI requests async | ALL LLM calls on background thread. NEVER block the render loop for AI. |

### Memory Budget
- Rect vertex buffer: pre-allocate 256 rects × 6 verts × 24 bytes = ~37 KB
- Display string: pre-allocate capacity for 80 cols × 60 lines = ~5 KB
- Glyphon buffer: managed by glyphon, typically < 1 MB
- AI conversation history: cap at 100 messages, evict oldest on overflow

---

## TASK 11: Living Organism Intelligence 🧬

**Forge is not a dead tool — it observes, learns, and evolves.**

### Integration Points (wire existing crates)

#### `forge-confidence` → Status Bar Confidence Score

```rust
// In status bar, show real-time confidence score
// "🧬 94.2%" — confidence that current code is correct
// Updates on every keystroke by feeding change context to ConfidenceEngine
```

#### `forge-surfaces` → UI Surface Intelligence

Every UI surface emits and receives confidence signals:
- **Gutter**: Color-code line numbers by confidence (green=high, yellow=medium, red=low)
- **Tab bar**: Tab background subtly tints based on file confidence
- **Status bar**: Overall file confidence score
- **Editor**: Current line confidence affects cursor color subtly
- **AI Panel**: Confidence score of AI suggestions shown inline

#### `forge-immune` → Self-Healing

```rust
// If an operation would corrupt the buffer:
// 1. forge-immune validates the transaction BEFORE applying
// 2. If suspicious, clone buffer state first (snapshot)
// 3. Apply transaction
// 4. If buffer invariants violated, rollback to snapshot
// 5. Log anomaly to forge-feedback
```

#### `forge-anticipation` → Predictive Behavior

```rust
// Pre-compute likely next actions:
// - If user is typing a function, pre-shape the closing brace line
// - If user is scrolling down, pre-render 2 screens ahead
// - If user opened file X, predict file Y will be opened next (warm cache)
// - Pre-warm AI context with likely questions based on cursor location
```

#### `forge-feedback` → Telemetry Loop

```rust
// Every user action records:
// - What was done (keystroke, scroll, save, etc.)
// - Time taken
// - Context (cursor position, file type, buffer size)
// - AI interactions (accepted/rejected suggestions)
// Feed this back to forge-confidence to improve predictions
```

### Organism Heartbeat

```rust
// Every 100ms, the organism "breathes":
// 1. Check buffer health (forge-immune)
// 2. Update confidence scores (forge-confidence)
// 3. Anticipate next action (forge-anticipation)
// 4. Propagate signals (forge-propagation)
// 5. Check AI agent health (forge-agent connection status)
// This runs on a background thread, never blocks the UI.
```

---

## TASK 12: Integrate Everything in `application.rs`

Rewrite the render function and event handler to use ALL of the above:

### Render Order (per frame)
1. `rect_renderer.clear()`
2. Compute layout from window size (including AI panel state)
3. Add rects: activity bar, tab bar, breadcrumbs, gutter, editor bg, status bar, current line highlight, cursor, selection rects, AI panel bg (if open)
4. `rect_renderer.upload()` — one GPU upload
5. Build text areas: tab labels, activity icons, line numbers, code text, status bar text, breadcrumbs, AI chat messages (if panel open)
6. `text_renderer.prepare()` with all text areas — one prepare call
7. Begin render pass (clear to editor bg color)
8. `rect_renderer.render()` — draw all rects
9. `text_renderer.render()` — draw all text on top
10. End render pass, submit, present

### Event Handler Updates
- Keyboard events: offset by activity bar width and tab bar height
- Mouse events: hit-test against layout zones (including AI panel)
- Scroll events: only in editor zone (or AI panel if focused)
- Click on tab: switch active tab
- Click on activity bar icon: toggle sidebar / AI panel
- `Ctrl+Shift+I` or `Ctrl+L`: toggle AI panel
- `Enter` in AI input: send message to agent

---

## TASK 13: Built-in AI Agent 🤖

**Forge has a brain. The AI agent is the developer's pair programmer, built directly into the editor.**

### Create `forge-agent` crate

Add to workspace `Cargo.toml`:
```toml
[workspace.dependencies]
reqwest = { version = "0.12", features = ["json", "stream"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["rt-multi-thread", "macros", "sync", "time"] }
```

### Architecture

```rust
// forge-agent/src/lib.rs

/// Multi-provider LLM agent that lives inside Forge
pub struct ForgeAgent {
    provider: Box<dyn LlmProvider>,
    conversation: Vec<Message>,
    system_prompt: String,
    /// Connection to editor state for context-aware responses
    editor_context: Arc<RwLock<EditorContext>>,
    /// Channel to send responses back to UI thread
    response_tx: mpsc::Sender<AgentEvent>,
}

/// What the agent can see about the editor
pub struct EditorContext {
    pub file_path: Option<String>,
    pub file_content: String,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub selected_text: Option<String>,
    pub language: String,
    pub project_files: Vec<String>,
    pub confidence_score: f64,
}

/// Events the agent sends back to the UI
pub enum AgentEvent {
    StreamChunk(String),           // streaming token
    MessageComplete(String),       // full response
    InlineCompletion(String),      // ghost text suggestion
    CodeAction(CodeAction),        // apply edit to buffer
    Error(String),                 // error message
    StatusUpdate(AgentStatus),     // connection status
}

pub enum AgentStatus {
    Ready,
    Thinking,
    Streaming,
    Error(String),
    Disconnected,
}
```

### Multi-Provider LLM Support

```rust
// forge-agent/src/providers/mod.rs

pub trait LlmProvider: Send + Sync {
    fn name(&self) -> &str;
    fn send(&self, messages: &[Message], ctx: &EditorContext) -> Result<String>;
    fn stream(&self, messages: &[Message], ctx: &EditorContext) -> Result<Receiver<String>>;
    fn supports_streaming(&self) -> bool;
}

// Providers to implement:
pub mod openai;      // OpenAI GPT-4, GPT-4o
pub mod anthropic;   // Claude 3.5 Sonnet, Claude 4
pub mod google;      // Gemini 2.5 Pro, Gemini 2.5 Flash
pub mod ollama;      // Local models (Llama, Mistral, Qwen)
pub mod openrouter;  // OpenRouter (access to all models)
```

### Provider Configuration

```rust
// forge-agent/src/config.rs
// Read from ~/.forge/agent.toml

/// Example config:
/// ```toml
/// [agent]
/// default_provider = "anthropic"
/// 
/// [agent.anthropic]
/// api_key = "sk-ant-..."
/// model = "claude-sonnet-4-20250514"
/// 
/// [agent.openai]
/// api_key = "sk-..."
/// model = "gpt-4o"
/// 
/// [agent.ollama]
/// url = "http://localhost:11434"
/// model = "qwen2.5-coder:32b"
/// 
/// [agent.google]
/// api_key = "..."
/// model = "gemini-2.5-pro"
/// ```
```

### System Prompt (Context-Aware)

```rust
const SYSTEM_PROMPT: &str = r#"
You are Forge AI — a pair programming agent embedded directly inside the Forge code editor.

You have LIVE access to:
- The file currently open (content, path, language)
- Cursor position (line, column)
- Selected text (if any)
- Project file tree
- Code confidence score from forge-confidence engine

Your capabilities:
1. **Answer questions** about the code
2. **Explain** code at cursor position
3. **Suggest fixes** for errors
4. **Generate code** based on natural language
5. **Refactor** selected code
6. **Write tests** for functions
7. **Review** code and suggest improvements

Response rules:
- Be concise. You're in an IDE, not a chat app.
- When showing code, use fenced code blocks with language tags.
- When suggesting edits, show ONLY the changed lines with +/- diff markers.
- If you can apply an edit directly, emit a CodeAction.
- Reference line numbers when discussing code.
"#;
```

### Chat Panel UI (rendered in the right panel)

```
+------------------------------------------+
|  🤖 Forge AI              [provider ▾] × |
+------------------------------------------+
|                                          |
|  [user] What does this function do?      |
|                                          |
|  [forge] This function `parse_config`    |
|  reads the TOML config file at line 42   |
|  and returns a `Config` struct. It       |
|  handles three edge cases:               |
|  - Missing file → default config         |
|  - Malformed TOML → error with line #    |
|  - Missing required fields → warnings    |
|                                          |
|  Confidence: 97.2%                       |
|                                          |
+------------------------------------------+
|  Ctrl+L to focus | Type message...    ⏎  |
+------------------------------------------+
```

### Panel Layout
- Width: 35% of window (min 300px, max 500px)
- Opens on right side, splits the editor area
- Drag-resizable border between editor and AI panel
- Scrollable message area
- Fixed input box at bottom (22px height)
- Header with provider name, model selector, close button

### Agent Commands (slash commands in input)

| Command | Action |
|---------|--------|
| `/explain` | Explain code at cursor or selection |
| `/fix` | Suggest fix for error at cursor |
| `/test` | Generate tests for function at cursor |
| `/refactor` | Refactor selected code |
| `/doc` | Generate documentation |
| `/review` | Review current file |
| `/ask <question>` | Free-form question about the code |
| `/model <name>` | Switch LLM provider/model |
| `/clear` | Clear conversation |

### Inline Ghost Text Completions

```rust
// When user pauses typing for 500ms:
// 1. Send current line + surrounding context to LLM
// 2. Receive completion suggestion
// 3. Render as ghost text (50% opacity) after cursor
// 4. Tab to accept, Escape to dismiss
// 5. Cancel pending completion on any keystroke

pub struct InlineCompletion {
    text: String,
    line: usize,
    col: usize,
    visible: bool,
}
```

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+Shift+I` | Toggle AI panel |
| `Ctrl+L` | Focus AI input |
| `Ctrl+Shift+E` | Explain selection |
| `Ctrl+Shift+F` | Fix error at cursor |
| `Tab` (when ghost text visible) | Accept inline completion |
| `Escape` | Dismiss ghost text / unfocus AI panel |
| `Enter` (in AI input) | Send message |
| `Shift+Enter` (in AI input) | New line in message |
| `Up/Down` (in AI input, empty) | Navigate message history |

### Background Threading

```rust
// ALL AI operations happen on a separate tokio runtime.
// The UI thread NEVER waits for AI.

// Architecture:
// UI Thread ──request_tx──→ Agent Thread (tokio) ──response_tx──→ UI Thread
//                             │
//                             ├─→ HTTP request to LLM API
//                             ├─→ Stream response chunks
//                             └─→ Send AgentEvent back to UI

pub struct AgentHandle {
    request_tx: mpsc::Sender<AgentRequest>,
    response_rx: mpsc::Receiver<AgentEvent>,
    runtime: tokio::runtime::Runtime,
}
```

### Agent-Editor Integration

```rust
// The agent can apply edits directly to the buffer:
pub struct CodeAction {
    pub description: String,
    pub edits: Vec<TextEdit>,
    pub preview: bool,  // show diff before applying
}

pub struct TextEdit {
    pub start_line: usize,
    pub end_line: usize,
    pub new_text: String,
}

// When agent sends a CodeAction:
// 1. Show diff in AI panel (red/green lines)
// 2. "Apply" button to accept
// 3. On accept: create Transaction, apply to buffer
// 4. Ctrl+Z undoes the AI edit (standard undo)
```

### Connection to Living Organism

The AI agent is wired into the organism's nervous system:

```rust
// forge-agent talks to forge-confidence:
// - Before sending to LLM: include confidence context
//   "The confidence engine rates this code at 73.4%. 
//    Lines 45-52 have the lowest confidence."
// - After receiving suggestion: run through forge-immune
//   to validate the suggestion won't corrupt the buffer

// forge-agent talks to forge-anticipation:
// - Predict when user is likely to ask for AI help
//   (e.g., after encountering a compile error)
// - Pre-warm the LLM context to reduce latency

// forge-agent talks to forge-feedback:
// - Log: did user accept or reject the suggestion?
// - Log: how long did the AI take to respond?
// - Feed acceptance rate back to improve system prompt
```

### Acceptance Criteria
- [ ] AI panel toggles open/closed with `Ctrl+Shift+I`
- [ ] User can type message and get response from LLM
- [ ] Streaming responses render token-by-token
- [ ] Agent has context: current file, cursor position, selection
- [ ] At least 2 providers work (Ollama for local + one cloud API)
- [ ] Inline ghost text completions appear after 500ms pause
- [ ] Tab accepts completion, Escape dismisses
- [ ] Slash commands work: `/explain`, `/fix`, `/test`
- [ ] CodeActions can apply edits to buffer (undoable)
- [ ] AI never blocks the UI — all requests are async
- [ ] Agent status shown in status bar
- [ ] Conversation history preserved per session
- [ ] Graceful handling of API errors, timeouts, rate limits

---

## TASK 14: Extension Store 🧩

**Forge has its own extension ecosystem — installable plugins that extend every surface.**

### Create `forge-extensions` crate

```rust
// forge-extensions/src/lib.rs

/// Extension manifest (each extension ships a forge-ext.toml)
pub struct ExtensionManifest {
    pub id: String,              // "forge-ext.rust-analyzer"
    pub name: String,            // "Rust Analyzer"
    pub version: String,         // "0.4.2"
    pub author: String,
    pub description: String,
    pub icon: Option<String>,    // base64 PNG or path
    pub entry_point: String,     // WASM module path
    pub permissions: Vec<Permission>,
    pub contributes: Contributions,
}

pub enum Permission {
    ReadBuffer,       // read file content
    WriteBuffer,      // modify file content
    FileSystem,       // read/write project files
    Network,          // make HTTP requests
    Terminal,         // spawn terminal commands
    Ui,               // add UI elements
}

pub struct Contributions {
    pub commands: Vec<Command>,
    pub keybindings: Vec<Keybinding>,
    pub languages: Vec<LanguageContribution>,
    pub themes: Vec<ThemeContribution>,
    pub activity_bar_icons: Vec<ActivityBarIcon>,
}
```

### Extension Runtime (Sandboxed WASM)

```rust
// Extensions run as WASM modules in a sandboxed runtime
// Using wasmtime for safe execution with capability-based permissions

pub struct ExtensionHost {
    runtime: wasmtime::Engine,
    extensions: Vec<LoadedExtension>,
    /// Channel to send extension events to UI
    event_tx: mpsc::Sender<ExtensionEvent>,
}

pub struct LoadedExtension {
    manifest: ExtensionManifest,
    instance: wasmtime::Instance,
    state: ExtensionState,
}

pub enum ExtensionState {
    Active,
    Disabled,
    Error(String),
}
```

### Extension Store UI Panel

```
+------------------------------------------+
|  🧩 Extensions              [⟳ Refresh] |
+------------------------------------------+
|  🔍 Search extensions...                 |
+------------------------------------------+
|  INSTALLED                               |
|  ✅ Rust Analyzer         v0.4.2  ⚙ ❌  |
|  ✅ Git Lens              v1.2.0  ⚙ ❌  |
|  ✅ Forge Dark+ Theme     v1.0.0  ⚙ ❌  |
|                                          |
|  RECOMMENDED                             |
|  ⬇  Python Support        v2.1.0  ⭐4.8 |
|  ⬇  Markdown Preview      v1.3.0  ⭐4.6 |
|  ⬇  Docker Support        v0.9.0  ⭐4.5 |
+------------------------------------------+
```

### Extension API (what extensions can do)

| API | Description |
|-----|-------------|
| `forge.editor.getText()` | Read current buffer |
| `forge.editor.insertText(pos, text)` | Insert text at position |
| `forge.editor.getSelection()` | Get selected text |
| `forge.editor.setCursorPosition(line, col)` | Move cursor |
| `forge.ui.showNotification(msg)` | Show notification |
| `forge.ui.addStatusBarItem(text, align)` | Add to status bar |
| `forge.ui.addActivityBarIcon(icon, panel)` | Add sidebar icon |
| `forge.commands.register(name, handler)` | Register command |
| `forge.languages.registerProvider(lang, provider)` | Language support |
| `forge.confidence.getScore()` | Read confidence score |
| `forge.agent.sendMessage(msg)` | Talk to AI agent |

### Extension Store Backend

```rust
// Extensions are distributed as .fext packages (ZIP with WASM + manifest)
// Store can be:
// 1. Built-in curated list (shipped with Forge)
// 2. Remote registry (future: forge-extensions.io)
// 3. Local .fext files (drag and drop to install)

pub struct ExtensionRegistry {
    local_store: PathBuf,           // ~/.forge/extensions/
    remote_url: Option<String>,     // https://extensions.forge.dev/api
    installed: Vec<ExtensionManifest>,
    available: Vec<ExtensionManifest>,
}

impl ExtensionRegistry {
    pub fn install(&mut self, id: &str) -> Result<()>;
    pub fn uninstall(&mut self, id: &str) -> Result<()>;
    pub fn update(&mut self, id: &str) -> Result<()>;
    pub fn search(&self, query: &str) -> Vec<&ExtensionManifest>;
}
```

### Acceptance Criteria

- [ ] Extension panel opens from activity bar (🧩 icon)
- [ ] Can list installed extensions
- [ ] Can install a local .fext file
- [ ] Extensions run in WASM sandbox (cannot crash Forge)
- [ ] Extension API provides read access to editor state
- [ ] At least one built-in extension works (e.g., word count)
- [ ] Extensions survive restart (persisted in ~/.forge/extensions/)
- [ ] Broken extension is disabled, not crashed

---

## TASK 15: Adaptive UI Modes 🎯

**The UI rearranges itself based on what the developer is doing.**

### Create `forge-app/src/modes.rs`

```rust
/// UI modes that rearrange the entire layout for maximum effectiveness
#[derive(Clone, Copy, PartialEq)]
pub enum UiMode {
    /// Standard VS Code layout — all panels available
    Standard,
    /// Focus mode — editor only, everything else hidden
    Focus,
    /// Performance mode — minimal UI, max frame budget for code
    Performance,
    /// Debug mode — split editor + terminal + variables + call stack
    Debug,
    /// Zen mode — centered editor, no chrome, just code
    Zen,
    /// Review mode — side-by-side diff + AI review panel
    Review,
}
```

### Mode Layouts

#### 🖥️ Standard Mode (default)
```
+----+-------------------------------------------+
| AB | Tabs | Breadcrumbs                         |
|    +-------------------------------------------+
|    | Gutter | Editor                  | AI Panel|
+----+-------------------------------------------+
| Status Bar                                     |
+------------------------------------------------+
```
- All panels available
- Activity bar, sidebar, tabs, status bar
- Keyboard: `Ctrl+Shift+P` → command palette

#### 🎯 Focus Mode
```
+------------------------------------------------+
| filename.rs                                     |
+------------------------------------------------+
|                                                 |
|          Gutter | Editor (centered, 80ch max)   |
|                                                 |
+------------------------------------------------+
```
- **Hides**: activity bar, sidebar, tabs, breadcrumbs, AI panel
- **Shows**: single tab header, centered editor (max 80-100 chars wide), gutter
- **Status bar**: minimal (line/col only)
- Keyboard: `Ctrl+Shift+F` to toggle
- Timer optional: "Focus for 25 min" (Pomodoro)

#### ⚡ Performance Mode
```
+------------------------------------------------+
|  Gutter | Editor (full width)                   |
|         |                                       |
+------------------------------------------------+
| ⚡ 0.3ms | Ln 42, Col 7                         |
+------------------------------------------------+
```
- **Hides**: activity bar, sidebar, tabs, breadcrumbs, AI panel
- **Disables**: cursor blink, smooth scroll, animations
- **Enables**: raw input, zero-overhead rendering
- **Status bar**: frame time + cursor position only
- Keyboard: `Ctrl+Shift+H` to toggle
- All organism heartbeat reduced to 500ms
- AI inline completions disabled

#### 🐛 Debug Mode
```
+----+-------------------+----------------------+
| AB | Editor            | Variables            |
|    |                   | > local_x: 42       |
|    |                   | > buffer: [...]      |
|    +-------------------+----------------------+
|    | Terminal / Output  | Call Stack           |
|    | > cargo run        | main() → parse()    |
|    | > error[E0308]...  | → validate()        |
+----+-------------------+----------------------+
| 🐛 DEBUGGING | main.rs:42 | ▶ Continue | ⏸ Pause |
+------------------------------------------------+
```
- **Shows**: editor (top-left), variables (top-right), terminal (bottom-left), call stack (bottom-right)
- **Status bar**: debug controls (continue, step, pause, stop)
- **Activity bar**: debug icon highlighted
- Breakpoint gutter markers (red dots)
- Keyboard: `F5` to start debugging, `F10` step over, `F11` step into

#### 🧘 Zen Mode
```
+------------------------------------------------+
|                                                 |
|                                                 |
|            fn main() {                          |
|                println!("hello");               |
|            }                                    |
|                                                 |
|                                                 |
+------------------------------------------------+
```
- **Hides**: EVERYTHING except the code
- No gutter, no status bar, no chrome
- Full-screen, centered text (60-80 chars)
- Subtle vignette gradient at edges
- `Escape` exits Zen mode
- Keyboard: `Ctrl+K Z` to toggle (VS Code compatible)

#### 📝 Review Mode
```
+----+-------------------+----------------------+
| AB | Original          | Modified             |
|    | - old_function()  | + new_function()     |
|    |   unchanged       |   unchanged          |
|    +-------------------+----------------------+
|    | 🤖 AI Review Comments                    |
|    | Line 14: Consider using `Option` here    |
|    | Line 28: This allocation can be avoided  |
+----+------------------------------------------+
| Status: 3 suggestions | 1 approved | 2 pending|
+------------------------------------------------+
```
- **Shows**: side-by-side diff (left=original, right=modified)
- AI review comments below
- Accept/reject buttons per suggestion
- Keyboard: `Ctrl+Shift+R` to toggle

### Mode Switching

```rust
impl UiMode {
    /// Keyboard shortcut for each mode
    pub fn shortcut(&self) -> &str {
        match self {
            Self::Standard => "Ctrl+Shift+P → 'Standard Mode'",
            Self::Focus => "Ctrl+Shift+F",
            Self::Performance => "Ctrl+Shift+H",
            Self::Debug => "F5",
            Self::Zen => "Ctrl+K Z",
            Self::Review => "Ctrl+Shift+R",
        }
    }

    /// Layout configuration for each mode
    pub fn layout_config(&self) -> LayoutConfig {
        match self {
            Self::Standard => LayoutConfig {
                activity_bar: true,
                tab_bar: true,
                breadcrumbs: true,
                gutter: true,
                status_bar: true,
                ai_panel: false,  // toggleable
                sidebar: false,   // toggleable
                center_editor: false,
                max_editor_width: None,
            },
            Self::Focus => LayoutConfig {
                activity_bar: false,
                tab_bar: true,   // single tab only
                breadcrumbs: false,
                gutter: true,
                status_bar: true, // minimal
                ai_panel: false,
                sidebar: false,
                center_editor: true,
                max_editor_width: Some(800.0),
            },
            // ... etc
        }
    }
}
```

### Mode Indicator in Status Bar

```
| 🖥️ Standard | or | 🎯 Focus | or | ⚡ Perf | or | 🐛 Debug | or | 🧘 Zen |
```

Click the mode indicator to cycle through modes, or use keyboard shortcuts.

### Smart Mode Switching (organism-driven)

```rust
// forge-anticipation can suggest mode switches:
// - Opened a debugger? → suggest Debug mode
// - No input for 30s while reading? → suggest Focus mode
// - Compile error? → suggest Debug mode
// - Writing docs? → suggest Zen mode
// - Reviewing PR? → suggest Review mode
// Show subtle notification: "Switch to Focus mode? [Y/n]"
```

### Acceptance Criteria

- [ ] All 6 modes render correctly
- [ ] Keyboard shortcuts switch between modes instantly
- [ ] Mode indicator visible in status bar
- [ ] Focus mode centers editor with max width
- [ ] Zen mode is truly full-screen, no chrome
- [ ] Performance mode disables all non-essential rendering
- [ ] Debug mode shows 4-panel layout
- [ ] Review mode shows side-by-side diff
- [ ] Mode state persists across sessions
- [ ] Transition between modes is smooth (no flicker)
- [ ] Organism can suggest mode switches

---

## TASK 16: Next-Gen Network Layer 🌐

**Forge's internet connection is unkillable. It auto-retries, queues offline, and uses the fastest protocols available.**

### Create `forge-net` crate

```rust
// forge-net/src/lib.rs

/// Zero-downtime network layer with automatic failover and retry
pub struct ForgeNet {
    client: reqwest::Client,
    connection_state: Arc<RwLock<ConnectionState>>,
    /// Offline request queue — requests made while disconnected
    offline_queue: Arc<Mutex<VecDeque<QueuedRequest>>>,
    /// Background health checker
    health_tx: mpsc::Sender<HealthEvent>,
    config: NetConfig,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ConnectionState {
    /// Full connectivity, all systems go
    Online { latency_ms: u32 },
    /// Degraded — some requests failing, auto-retrying
    Degraded { error_rate: f32 },
    /// Offline — queuing requests for replay when back online
    Offline { since: Instant, queued: usize },
    /// Reconnecting — actively trying to restore connection
    Reconnecting { attempt: u32, next_retry_ms: u64 },
}

pub struct NetConfig {
    /// Maximum concurrent connections per host
    pub max_connections_per_host: usize,  // default: 20
    /// Connection timeout
    pub connect_timeout: Duration,        // default: 5s
    /// Request timeout
    pub request_timeout: Duration,        // default: 30s
    /// Maximum retry attempts before going offline
    pub max_retries: u32,                 // default: 5
    /// Base retry delay (exponential backoff)
    pub base_retry_delay: Duration,       // default: 100ms
    /// Maximum retry delay cap
    pub max_retry_delay: Duration,        // default: 30s
    /// Enable HTTP/2 multiplexing
    pub http2_adaptive_window: bool,      // default: true
    /// Offline queue capacity
    pub offline_queue_size: usize,        // default: 1000
    /// Health check interval
    pub health_check_interval: Duration,  // default: 5s
}
```

### Exponential Backoff with Jitter

```rust
// forge-net/src/retry.rs

/// Retry strategy: exponential backoff with decorrelated jitter
/// This is the theoretically optimal retry strategy for distributed systems
/// (see AWS Architecture Blog: "Exponential Backoff And Jitter")
pub struct RetryPolicy {
    base_delay: Duration,
    max_delay: Duration,
    max_attempts: u32,
    jitter: JitterStrategy,
}

pub enum JitterStrategy {
    /// Full jitter: random(0, min(cap, base * 2^attempt))
    /// Best for reducing thundering herd
    Full,
    /// Decorrelated jitter: min(cap, random(base, prev_delay * 3))
    /// Best overall performance (AWS recommended)
    Decorrelated,
    /// Equal jitter: half exponential + half random
    Equal,
}

impl RetryPolicy {
    pub fn next_delay(&self, attempt: u32, prev_delay: Duration) -> Duration {
        match self.jitter {
            JitterStrategy::Decorrelated => {
                let max_ms = self.max_delay.as_millis() as u64;
                let base_ms = self.base_delay.as_millis() as u64;
                let prev_ms = prev_delay.as_millis() as u64;
                let delay = rand::thread_rng()
                    .gen_range(base_ms..=(prev_ms.saturating_mul(3)).min(max_ms));
                Duration::from_millis(delay)
            }
            // ... other strategies
        }
    }

    pub fn should_retry(&self, attempt: u32, error: &NetError) -> bool {
        attempt < self.max_attempts && error.is_retryable()
    }
}

/// Errors classified by retryability
impl NetError {
    pub fn is_retryable(&self) -> bool {
        matches!(self,
            NetError::Timeout |
            NetError::ConnectionReset |
            NetError::ServerError(500..=599) |
            NetError::TooManyRequests |  // 429 — respect Retry-After header
            NetError::DnsResolution |
            NetError::NetworkUnreachable
        )
    }
}
```

### Offline Queue & Replay

```rust
// forge-net/src/offline.rs

/// When connection drops, requests are queued and replayed on reconnect
pub struct OfflineQueue {
    queue: VecDeque<QueuedRequest>,
    max_size: usize,
    /// Priority ordering: critical requests first
    priority_order: bool,
}

pub struct QueuedRequest {
    pub url: String,
    pub method: Method,
    pub body: Option<Vec<u8>>,
    pub headers: HeaderMap,
    pub priority: RequestPriority,
    pub queued_at: Instant,
    pub max_age: Duration,  // expired requests are dropped
    pub callback: oneshot::Sender<Result<Response>>,
}

pub enum RequestPriority {
    /// Must be sent: save file, commit, push
    Critical,
    /// Should be sent: AI responses, extension updates
    Normal,
    /// Nice to have: telemetry, analytics, extension store browse
    Low,
}

impl OfflineQueue {
    /// Replay all queued requests when connection restores
    pub async fn replay(&mut self, client: &reqwest::Client) {
        // Sort by priority (Critical first)
        // Drop expired requests
        // Send in order, respecting rate limits
        // Failed replays go back to queue (with retry count)
    }
}
```

### Connection Health Monitor

```rust
// forge-net/src/health.rs

/// Background task that monitors connection health
pub struct HealthMonitor {
    /// Ping targets to check connectivity
    check_urls: Vec<String>,
    /// Rolling window of request success/failure
    history: VecDeque<RequestResult>,
    /// Current state
    state: ConnectionState,
}

impl HealthMonitor {
    /// Runs every 5 seconds on background thread
    pub async fn check(&mut self) -> ConnectionState {
        // 1. Try HEAD request to known endpoint (e.g., forge API)
        // 2. Measure latency
        // 3. If failed, try fallback endpoints
        // 4. Update rolling success rate
        // 5. Transition state machine:
        //    Online → Degraded (if error_rate > 10%)
        //    Degraded → Offline (if error_rate > 50%)
        //    Offline → Reconnecting (periodic retry)
        //    Reconnecting → Online (if health check passes)
    }
}
```

### HTTP/2 + HTTP/3 Protocol Optimization

```rust
// forge-net/src/client.rs

impl ForgeNet {
    pub fn new(config: NetConfig) -> Self {
        let client = reqwest::Client::builder()
            // Connection pooling
            .pool_max_idle_per_host(config.max_connections_per_host)
            .pool_idle_timeout(Duration::from_secs(90))
            // HTTP/2
            .http2_adaptive_window(config.http2_adaptive_window)
            .http2_keep_alive_interval(Duration::from_secs(20))
            .http2_keep_alive_timeout(Duration::from_secs(5))
            // Timeouts
            .connect_timeout(config.connect_timeout)
            .timeout(config.request_timeout)
            // DNS
            .trust_dns(true)  // Use trust-dns for faster resolution
            // Compression
            .gzip(true)
            .brotli(true)
            .zstd(true)
            // TLS
            .use_rustls_tls()
            .min_tls_version(reqwest::tls::Version::TLS_1_2)
            // TCP optimization
            .tcp_nodelay(true)
            .tcp_keepalive(Duration::from_secs(60))
            .build()
            .expect("Failed to build HTTP client");

        Self { client, .. }
    }

    /// Send request with automatic retry and offline queueing
    pub async fn request(&self, req: Request) -> Result<Response> {
        let mut attempt = 0;
        let mut prev_delay = self.config.base_retry_delay;

        loop {
            match self.client.execute(req.try_clone()?).await {
                Ok(resp) if resp.status().is_success() => {
                    self.update_state(ConnectionState::Online {
                        latency_ms: elapsed.as_millis() as u32
                    });
                    return Ok(resp);
                }
                Ok(resp) if resp.status() == 429 => {
                    // Rate limited — respect Retry-After header
                    let retry_after = resp.headers()
                        .get("retry-after")
                        .and_then(|v| v.to_str().ok())
                        .and_then(|v| v.parse::<u64>().ok())
                        .unwrap_or(1);
                    tokio::time::sleep(Duration::from_secs(retry_after)).await;
                    continue;
                }
                Err(e) if RetryPolicy::default().should_retry(attempt, &e.into()) => {
                    attempt += 1;
                    let delay = self.retry_policy.next_delay(attempt, prev_delay);
                    prev_delay = delay;
                    self.update_state(ConnectionState::Degraded {
                        error_rate: attempt as f32 / self.config.max_retries as f32
                    });
                    tokio::time::sleep(delay).await;
                }
                Err(e) => {
                    // All retries exhausted — queue for later if important
                    self.queue_request(req, RequestPriority::Normal).await;
                    self.update_state(ConnectionState::Offline {
                        since: Instant::now(),
                        queued: self.offline_queue.lock().len(),
                    });
                    return Err(e.into());
                }
                Ok(resp) => return Ok(resp),  // non-success but non-retryable
            }
        }
    }
}
```

### Status Bar Network Indicator

```
| 🌐 Online 12ms | or | ⚠️ Degraded 340ms | or | 🔴 Offline (3 queued) | or | 🔄 Reconnecting... |
```

- **Online**: green, shows latency
- **Degraded**: yellow, shows elevated latency
- **Offline**: red, shows queued request count
- **Reconnecting**: spinning, shows attempt number

### Integration with AI Agent

```rust
// forge-agent uses forge-net for ALL HTTP requests:
// - LLM API calls go through ForgeNet.request()
// - If offline, AI gracefully degrades:
//   1. Show "Offline — queued" in chat
//   2. When back online, send queued message and stream response
//   3. Meanwhile, offer local Ollama if available

// Extension store uses forge-net:
// - Extension downloads go through ForgeNet
// - If offline, show cached extension list
// - Queue install requests for when back online
```

### Acceptance Criteria

- [ ] All HTTP requests go through `ForgeNet`
- [ ] Auto-retry with exponential backoff + decorrelated jitter
- [ ] Requests queue when offline, replay when back online
- [ ] Connection state visible in status bar with latency
- [ ] Health monitor detects connectivity changes within 5s
- [ ] HTTP/2 connection pooling and multiplexing enabled
- [ ] Compression enabled (gzip, brotli, zstd)
- [ ] Rate limit responses (429) respected with Retry-After
- [ ] AI agent gracefully degrades to offline mode
- [ ] No request silently dropped — all queued or error-reported
- [ ] Queue respects priority ordering and TTL expiry
- [ ] Network layer never blocks the UI thread

---

## Build & Verification

```bash
# Must pass
cargo check --package forge-app
cargo check --package forge-agent
cargo check --package forge-net
cargo build --release --package forge-app
cargo test --workspace

# Must not crash
./target/release/forge                          # empty buffer
./target/release/forge Cargo.toml               # open a file
./target/release/forge src/main.rs              # open own source
# Resize window rapidly
# Scroll to end of file and back
# Type rapidly for 10 seconds
# Ctrl+Z spam (undo 100 times)
# Open a 10,000 line file
# Toggle AI panel rapidly
# Send AI message with no API key configured
# Send AI message with invalid API key
# Cycle through all UI modes rapidly
# Open extension panel
# Disable network adapter → verify offline detection
# Re-enable network → verify queue replay
```

### Visual Checklist

- [ ] Activity bar visible on left (dark, 48px)
- [ ] Tab bar with filename tab visible
- [ ] Breadcrumb bar visible
- [ ] Line numbers in gutter
- [ ] Current line highlighted
- [ ] Cursor visible and blinking
- [ ] Status bar blue at bottom with Ln/Col
- [ ] Confidence score in status bar
- [ ] AI agent status in status bar
- [ ] Network connection indicator in status bar
- [ ] AI panel opens/closes smoothly
- [ ] AI chat renders messages with correct styling
- [ ] Inline ghost text completions appear
- [ ] Extension panel lists installed extensions
- [ ] Mode indicator in status bar
- [ ] Focus mode centers editor
- [ ] Zen mode hides all chrome
- [ ] No visual glitches on resize
- [ ] Smooth scrolling
- [ ] Frame time < 7ms (shown in status bar)

---

## Critical Rules for Jules

1. **`cargo check` must pass with ZERO errors after every file change.** Do NOT proceed to the next task if the build is broken.
2. **Use `wgpu = "23"` — NOT 24.** glyphon 0.7 requires wgpu 23. Do not upgrade.
3. **Use `wgpu::Instance::new(InstanceDescriptor { ... })` — by value, NOT reference.** wgpu 23 API.
4. **All rendering state (`text_atlas`, `text_renderer`) must be `mut` where needed.**
5. **Never `unwrap()` in render path.** Use `if let`, `match`, or `.unwrap_or_default()`.
6. **Never allocate in the hot render loop.** Pre-allocate and reuse.
7. **Run `cargo test --workspace` before marking done.**
8. **The organism heartbeat runs on a BACKGROUND THREAD — never block the event loop.**
9. **ALL AI/LLM calls run on a background tokio runtime — never block the UI for AI.**
10. **ALL network requests go through `ForgeNet` — never use raw reqwest directly.**
11. **Extensions run in WASM sandbox — a broken extension must NEVER crash Forge.**
12. **If a task is too complex, split into smaller commits. Never ship a broken build.**
13. **When in doubt, skip the feature and leave a `// TODO:` comment. A working editor > a crashed editor.**
14. **AI agent must gracefully handle: no API key, network errors, malformed responses, timeouts.**
15. **Mode switches must be instant (< 1 frame). Pre-compute all mode layouts at startup.**
16. **Network layer must auto-recover from ANY connectivity loss. No request silently dropped.**
