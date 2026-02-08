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
}

impl Layout {
    pub fn compute(width: f32, height: f32) -> Self;
}
```

### Acceptance Criteria
- [ ] All zones tile perfectly with zero gaps and zero overlap at any window size
- [ ] Minimum window size clamped to 400×300 — no layout collapse
- [ ] Layout recomputes instantly on resize, no allocation

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
| Settings (bottom) | Settings | `⚙` |

### Behavior
- Each icon is 48×48 centered in the bar
- Active icon: white, left border (2px #007acc)
- Inactive icon: gray (60% opacity)
- Click toggles sidebar (sidebar can start as hidden)

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
- **Right side:** `Ln {line}, Col {col}` | `Spaces: 4` | `UTF-8` | `Rust` | `🔥 Forge`

### Implementation
- Use `RectRenderer` for blue background
- Use `glyphon` TextArea for status text
- Update on every cursor move and file change

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

### Recovery Strategy
- On ANY render error: skip that frame, request redraw next frame
- On file error: show error text in status bar for 3 seconds
- On GPU error: attempt surface reconfiguration up to 3 times

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

### Memory Budget
- Rect vertex buffer: pre-allocate 256 rects × 6 verts × 24 bytes = ~37 KB
- Display string: pre-allocate capacity for 80 cols × 60 lines = ~5 KB
- Glyphon buffer: managed by glyphon, typically < 1 MB

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
```

#### `forge-feedback` → Telemetry Loop

```rust
// Every user action records:
// - What was done (keystroke, scroll, save, etc.)
// - Time taken
// - Context (cursor position, file type, buffer size)
// Feed this back to forge-confidence to improve predictions
```

### Organism Heartbeat

```rust
// Every 100ms, the organism "breathes":
// 1. Check buffer health (forge-immune)
// 2. Update confidence scores (forge-confidence)
// 3. Anticipate next action (forge-anticipation)
// 4. Propagate signals (forge-propagation)
// This runs on a background thread, never blocks the UI.
```

---

## TASK 12: Integrate Everything in `application.rs`

Rewrite the render function and event handler to use ALL of the above:

### Render Order (per frame)
1. `rect_renderer.clear()`
2. Compute layout from window size
3. Add rects: activity bar, tab bar, breadcrumbs, gutter, editor bg, status bar, current line highlight, cursor, selection rects
4. `rect_renderer.upload()` — one GPU upload
5. Build text areas: tab labels, activity icons, line numbers, code text, status bar text, breadcrumbs
6. `text_renderer.prepare()` with all text areas — one prepare call
7. Begin render pass (clear to editor bg color)
8. `rect_renderer.render()` — draw all rects
9. `text_renderer.render()` — draw all text on top
10. End render pass, submit, present

### Event Handler Updates
- Keyboard events: offset by activity bar width and tab bar height
- Mouse events: hit-test against layout zones
- Scroll events: only in editor zone
- Click on tab: switch active tab
- Click on activity bar icon: toggle sidebar (future)

---

## Build & Verification

```bash
# Must pass
cargo check --package forge-app
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
9. **If a task is too complex, split into smaller commits. Never ship a broken build.**
10. **When in doubt, skip the feature and leave a `// TODO:` comment. A working editor > a crashed editor.**
