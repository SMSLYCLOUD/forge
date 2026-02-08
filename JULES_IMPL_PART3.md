# JULES IMPLEMENTATION — PART 3 OF 4
# Tasks 9-12: Crash Guardrails, Turbo Performance, Organism Integration, Full Integration

> **CRITICAL**: Complete Parts 1 and 2 first. Run `cargo check` after every file change.

---

## TASK 9: Anti-Crash Guardrails

### Update file: `crates/forge-app/src/application.rs`

Wrap EVERY fallible operation in the render path with safe error handling.
Apply these rules everywhere in the render loop:

```rust
// RULE 1: Never unwrap() in render path
// BAD:
let surface_texture = surface.get_current_texture().unwrap();
// GOOD:
let surface_texture = match surface.get_current_texture() {
    Ok(texture) => texture,
    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
        // Reconfigure surface and skip this frame
        surface.configure(&device, &config);
        return;
    }
    Err(wgpu::SurfaceError::OutOfMemory) => {
        tracing::error!("GPU out of memory!");
        return;
    }
    Err(e) => {
        tracing::warn!("Surface error: {:?}, skipping frame", e);
        return;
    }
};

// RULE 2: Guard against zero-size windows
if config.width == 0 || config.height == 0 {
    return; // Don't render to zero-size surface
}

// RULE 3: Guard against index out of bounds in editor
let line_text = editor.buffer
    .line(line_idx)
    .map(|l| l.to_string())
    .unwrap_or_default();

// RULE 4: Clamp scroll position
editor.scroll_top = editor.scroll_top.min(
    editor.total_lines().saturating_sub(1)
);

// RULE 5: Guard against division by zero
let visible_lines = if line_height > 0.0 {
    (zone.height / line_height) as usize
} else {
    1
};
```

### Create file: `crates/forge-app/src/guard.rs`

```rust
/// Safe wrapper for fallible operations in the render path
pub struct Guard;

impl Guard {
    /// Safely get a line from the buffer, returning empty string if out of bounds
    pub fn get_line(buffer: &ropey::Rope, line_idx: usize) -> String {
        if line_idx < buffer.len_lines() {
            buffer.line(line_idx).to_string()
        } else {
            String::new()
        }
    }

    /// Clamp a value between min and max
    pub fn clamp_usize(value: usize, min: usize, max: usize) -> usize {
        value.max(min).min(max)
    }

    /// Safe division that returns default on zero divisor
    pub fn safe_div_f32(numerator: f32, denominator: f32, default: f32) -> f32 {
        if denominator.abs() < f32::EPSILON {
            default
        } else {
            numerator / denominator
        }
    }

    /// Clamp cursor to valid buffer position
    pub fn clamp_cursor(line: usize, col: usize, total_lines: usize, line_len: usize) -> (usize, usize) {
        let safe_line = if total_lines == 0 { 0 } else { line.min(total_lines - 1) };
        let safe_col = col.min(line_len);
        (safe_line, safe_col)
    }

    /// Safe slice of a string
    pub fn safe_substr(s: &str, start: usize, max_len: usize) -> &str {
        if start >= s.len() {
            ""
        } else {
            let end = (start + max_len).min(s.len());
            // Find valid char boundaries
            let start = s.floor_char_boundary(start);
            let end = s.ceil_char_boundary(end.min(s.len()));
            &s[start..end]
        }
    }
}
```

### Update module declarations

```rust
mod guard;
```

### Run `cargo check --package forge-app` — fix ALL errors.

---

## TASK 10: Turbo Performance Optimizations

### Update file: `crates/forge-app/src/application.rs`

Add frame timing and pre-allocation optimizations:

```rust
use std::time::{Duration, Instant};

/// Frame timing state
pub struct FrameTimer {
    last_frame: Instant,
    frame_times: [f32; 60], // Rolling window
    frame_index: usize,
    pub avg_frame_time_ms: f32,
}

impl FrameTimer {
    pub fn new() -> Self {
        Self {
            last_frame: Instant::now(),
            frame_times: [0.0; 60],
            frame_index: 0,
            avg_frame_time_ms: 0.0,
        }
    }

    pub fn begin_frame(&mut self) {
        self.last_frame = Instant::now();
    }

    pub fn end_frame(&mut self) {
        let elapsed = self.last_frame.elapsed().as_secs_f32() * 1000.0;
        self.frame_times[self.frame_index] = elapsed;
        self.frame_index = (self.frame_index + 1) % 60;
        self.avg_frame_time_ms = self.frame_times.iter().sum::<f32>() / 60.0;
    }
}

impl Default for FrameTimer {
    fn default() -> Self {
        Self::new()
    }
}

/// Pre-allocated render collections (avoid allocating in hot loop)
pub struct RenderBatch {
    /// Pre-allocated rectangle collection
    pub rects: Vec<crate::rect_renderer::Rect>,
}

impl RenderBatch {
    pub fn new() -> Self {
        Self {
            rects: Vec::with_capacity(2048),
        }
    }

    pub fn clear(&mut self) {
        self.rects.clear();
    }

    pub fn push(&mut self, rect: crate::rect_renderer::Rect) {
        self.rects.push(rect);
    }

    pub fn extend(&mut self, rects: &[crate::rect_renderer::Rect]) {
        self.rects.extend_from_slice(rects);
    }
}

impl Default for RenderBatch {
    fn default() -> Self {
        Self::new()
    }
}
```

### Scrollbar rendering (add to `crates/forge-app/src/scrollbar.rs`)

```rust
use crate::rect_renderer::Rect;
use crate::ui::{colors, LayoutConstants, Zone};

/// Vertical scrollbar state and rendering
pub struct Scrollbar {
    /// Whether mouse is over scrollbar
    pub hovered: bool,
    /// Whether scrollbar is being dragged
    pub dragging: bool,
    /// Drag start Y position
    drag_start_y: f32,
    drag_start_scroll: usize,
}

impl Scrollbar {
    pub fn new() -> Self {
        Self {
            hovered: false,
            dragging: false,
            drag_start_y: 0.0,
            drag_start_scroll: 0,
        }
    }

    /// Calculate scrollbar thumb dimensions
    fn thumb_geometry(&self, zone: &Zone, total_lines: usize, visible_lines: usize, scroll_top: usize) -> (f32, f32) {
        if total_lines <= visible_lines {
            return (zone.y, zone.height);
        }

        let ratio = visible_lines as f32 / total_lines as f32;
        let thumb_height = (zone.height * ratio).max(30.0); // Minimum 30px
        let scroll_ratio = scroll_top as f32 / (total_lines - visible_lines) as f32;
        let thumb_y = zone.y + scroll_ratio * (zone.height - thumb_height);

        (thumb_y, thumb_height)
    }

    /// Generate scrollbar rectangle
    pub fn render_rect(&self, zone: &Zone, total_lines: usize, visible_lines: usize, scroll_top: usize) -> Vec<Rect> {
        let mut rects = Vec::with_capacity(2);

        // Track background
        rects.push(Rect {
            x: zone.x,
            y: zone.y,
            width: zone.width,
            height: zone.height,
            color: [0.15, 0.15, 0.15, 0.3],
        });

        // Thumb
        let (thumb_y, thumb_height) = self.thumb_geometry(zone, total_lines, visible_lines, scroll_top);
        let thumb_color = if self.dragging {
            [0.5, 0.5, 0.5, 0.8]
        } else if self.hovered {
            [0.4, 0.4, 0.4, 0.7]
        } else {
            colors::SCROLLBAR
        };

        rects.push(Rect {
            x: zone.x + 2.0,
            y: thumb_y,
            width: zone.width - 4.0,
            height: thumb_height,
            color: thumb_color,
        });

        rects
    }

    /// Start dragging
    pub fn start_drag(&mut self, mouse_y: f32, scroll_top: usize) {
        self.dragging = true;
        self.drag_start_y = mouse_y;
        self.drag_start_scroll = scroll_top;
    }

    /// Update during drag, returns new scroll_top
    pub fn update_drag(&self, mouse_y: f32, zone: &Zone, total_lines: usize, visible_lines: usize) -> usize {
        if total_lines <= visible_lines {
            return 0;
        }

        let (_, thumb_height) = self.thumb_geometry(zone, total_lines, visible_lines, self.drag_start_scroll);
        let delta_y = mouse_y - self.drag_start_y;
        let scroll_range = zone.height - thumb_height;
        if scroll_range <= 0.0 {
            return 0;
        }

        let delta_scroll = (delta_y / scroll_range * (total_lines - visible_lines) as f32) as isize;
        let new_scroll = (self.drag_start_scroll as isize + delta_scroll).max(0) as usize;
        new_scroll.min(total_lines.saturating_sub(visible_lines))
    }

    /// Stop dragging
    pub fn stop_drag(&mut self) {
        self.dragging = false;
    }
}

impl Default for Scrollbar {
    fn default() -> Self {
        Self::new()
    }
}
```

### Update module declarations

```rust
mod scrollbar;
```

### Run `cargo check --package forge-app` — fix ALL errors.

---

## TASK 11: Living Organism Intelligence Integration

The organism layer connects to existing crates. Wire the existing `forge-confidence`, `forge-anticipation`, `forge-immune`, and `forge-feedback` crates into the UI.

### Create file: `crates/forge-app/src/organism.rs`

```rust
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Organism state that the UI reads from
/// This is updated by background threads; UI only reads
#[derive(Clone, Debug)]
pub struct OrganismState {
    /// Overall confidence score (0.0 - 100.0)
    pub confidence_score: f32,
    /// Per-line confidence (line_index → score)
    pub line_confidence: Vec<f32>,
    /// Anticipation predictions
    pub predictions: Vec<Prediction>,
    /// Last heartbeat time
    pub last_heartbeat: Instant,
    /// Whether the organism is running
    pub alive: bool,
}

#[derive(Clone, Debug)]
pub struct Prediction {
    pub action: String,
    pub probability: f32,
    pub pre_warm: bool,
}

impl OrganismState {
    pub fn new() -> Self {
        Self {
            confidence_score: 0.0,
            line_confidence: Vec::new(),
            predictions: Vec::new(),
            last_heartbeat: Instant::now(),
            alive: false,
        }
    }
}

impl Default for OrganismState {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe handle to organism state
pub type SharedOrganismState = Arc<Mutex<OrganismState>>;

/// Create a new shared organism state
pub fn new_shared_state() -> SharedOrganismState {
    Arc::new(Mutex::new(OrganismState::new()))
}

/// Start the organism heartbeat on a background thread
/// This thread periodically updates the organism state
pub fn start_heartbeat(state: SharedOrganismState, heartbeat_interval: Duration) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(heartbeat_interval);

            if let Ok(mut s) = state.lock() {
                s.last_heartbeat = Instant::now();
                s.alive = true;

                // TODO: Connect to actual forge-confidence crate
                // For now, set a default confidence score
                if s.confidence_score < 0.01 {
                    s.confidence_score = 75.0;
                }
            }
        }
    })
}

/// Read organism state safely (never blocks UI for more than a few microseconds)
pub fn read_state(state: &SharedOrganismState) -> Option<OrganismState> {
    // try_lock to avoid blocking the render thread
    state.try_lock().ok().map(|guard| guard.clone())
}
```

### Update module declarations

```rust
mod organism;
```

### Run `cargo check --package forge-app` — fix ALL errors.

---

## TASK 12: Full Integration in `application.rs`

### Update file: `crates/forge-app/src/application.rs`

This is the main integration task. You need to:

1. Create instances of all new components
2. Wire them into the event loop
3. Render everything in the correct order

Here is the complete structure for the `ForgeApplication`:

```rust
use crate::rect_renderer::RectRenderer;
use crate::ui::{LayoutZones, LayoutConstants};
use crate::tab_bar::TabBar;
use crate::activity_bar::ActivityBar;
use crate::gutter::Gutter;
use crate::status_bar::StatusBar;
use crate::cursor::CursorRenderer;
use crate::breadcrumb::BreadcrumbBar;
use crate::scrollbar::Scrollbar;
use crate::organism::{self, SharedOrganismState};
use crate::guard::Guard;

pub struct ForgeApplication {
    // GPU resources (already exist)
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,

    // Text rendering (already exists)
    text_atlas: glyphon::TextAtlas,
    text_renderer: glyphon::TextRenderer,
    font_system: glyphon::FontSystem,
    swash_cache: glyphon::SwashCache,

    // Editor state (already exists)
    editor: crate::editor::EditorState,

    // NEW: Rectangle renderer
    rect_renderer: RectRenderer,

    // NEW: UI components
    layout: LayoutZones,
    tab_bar: TabBar,
    activity_bar: ActivityBar,
    gutter: Gutter,
    status_bar_state: StatusBar,
    cursor_renderer: CursorRenderer,
    breadcrumb_bar: BreadcrumbBar,
    scrollbar: Scrollbar,

    // NEW: UI state
    sidebar_open: bool,
    ai_panel_open: bool,

    // NEW: Performance
    frame_timer: crate::application::FrameTimer,
    render_batch: crate::application::RenderBatch,

    // NEW: Organism
    organism_state: SharedOrganismState,
}
```

### Initialization (in `new()` or resume):

```rust
// After existing GPU setup...
let rect_renderer = RectRenderer::new(&device, config.format);

let layout = LayoutZones::compute(
    config.width as f32,
    config.height as f32,
    false, // sidebar closed initially
    false, // AI panel closed initially
);

let mut tab_bar = TabBar::new();
let activity_bar = ActivityBar::new();
let mut gutter = Gutter::new();
let status_bar_state = StatusBar::new();
let cursor_renderer = CursorRenderer::new();
let mut breadcrumb_bar = BreadcrumbBar::new();
let scrollbar = Scrollbar::new();
let frame_timer = FrameTimer::new();
let render_batch = RenderBatch::new();

// If a file was opened, update components
if let Some(path) = &file_path {
    let filename = std::path::Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "untitled".to_string());
    tab_bar.open_tab(filename, Some(path.clone()));
    breadcrumb_bar.update_from_path(path);
}

// Start organism heartbeat
let organism_state = organism::new_shared_state();
let _heartbeat = organism::start_heartbeat(
    organism_state.clone(),
    std::time::Duration::from_millis(250),
);
```

### Resize handler:

```rust
fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
    if new_size.width > 0 && new_size.height > 0 {
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);

        // Recalculate layout
        self.layout = LayoutZones::compute(
            new_size.width as f32,
            new_size.height as f32,
            self.sidebar_open,
            self.ai_panel_open,
        );

        // Update rect renderer uniform
        self.rect_renderer.resize(&self.queue, new_size.width as f32, new_size.height as f32);
    }
}
```

### Render loop (the critical integration):

```rust
fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
    self.frame_timer.begin_frame();

    // Guard: zero-size window
    if self.config.width == 0 || self.config.height == 0 {
        return Ok(());
    }

    let surface_texture = self.surface.get_current_texture()?;
    let view = surface_texture.texture.create_view(&wgpu::TextureViewDescriptor::default());

    // --- COLLECT ALL RECTANGLES (no allocations - reuse batch) ---
    self.render_batch.clear();

    // 1. Background rectangles (layout chrome)
    let bg_rects = self.layout.background_rects();
    self.render_batch.extend(&bg_rects);

    // 2. Tab bar
    let tab_rects = self.tab_bar.render_rects(&self.layout.tab_bar);
    self.render_batch.extend(&tab_rects);

    // 3. Activity bar
    let ab_rects = self.activity_bar.render_rects(&self.layout.activity_bar);
    self.render_batch.extend(&ab_rects);

    // 4. Gutter (line numbers + breakpoints)
    self.gutter.scroll_top = self.editor.scroll_top();
    self.gutter.total_lines = self.editor.total_lines();
    self.gutter.cursor_line = self.editor.cursor_line();
    let gutter_rects = self.gutter.render_rects(&self.layout.gutter);
    self.render_batch.extend(&gutter_rects);

    // 5. Current line highlight
    if let Some(hl_rect) = self.cursor_renderer.current_line_rect(
        self.editor.cursor_line(),
        self.editor.scroll_top(),
        &self.layout.editor,
    ) {
        self.render_batch.push(hl_rect);
    }

    // 6. Cursor
    self.cursor_renderer.update();
    if let Some(cursor_rect) = self.cursor_renderer.render_rect(
        self.editor.cursor_line(),
        self.editor.cursor_col(),
        self.editor.scroll_top(),
        &self.layout.editor,
    ) {
        self.render_batch.push(cursor_rect);
    }

    // 7. Scrollbar
    let visible_lines = (self.layout.editor.height / LayoutConstants::LINE_HEIGHT) as usize;
    let sb_rects = self.scrollbar.render_rect(
        &self.layout.scrollbar_v,
        self.editor.total_lines(),
        visible_lines,
        self.editor.scroll_top(),
    );
    self.render_batch.extend(&sb_rects);

    // 8. Breadcrumb separators
    let bc_rects = self.breadcrumb_bar.render_rects(&self.layout.breadcrumb_bar);
    self.render_batch.extend(&bc_rects);

    // --- UPLOAD & RENDER RECTANGLES ---
    self.rect_renderer.prepare(&self.queue, &self.render_batch.rects);

    // --- PREPARE TEXT ---
    // Update status bar with current state
    self.status_bar_state.cursor_line = self.editor.cursor_line() + 1;
    self.status_bar_state.cursor_col = self.editor.cursor_col() + 1;
    self.status_bar_state.frame_time_ms = self.frame_timer.avg_frame_time_ms;

    // Read organism state (non-blocking)
    if let Some(org_state) = organism::read_state(&self.organism_state) {
        self.status_bar_state.confidence_score = Some(org_state.confidence_score);
    }

    // TODO: Use glyphon to render text for:
    // - Tab titles (self.tab_bar.text_positions(&self.layout.tab_bar))
    // - Line numbers (self.gutter.text_positions(&self.layout.gutter))
    // - Status bar items (self.status_bar_state.text_positions(&self.layout.status_bar))
    // - Breadcrumbs (self.breadcrumb_bar.text_positions(&self.layout.breadcrumb_bar))
    // - Activity bar icons (self.activity_bar.text_positions(&self.layout.activity_bar))
    // - Editor text (existing text rendering logic)

    // --- GPU RENDER PASS ---
    let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Render Encoder"),
    });

    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Main Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.118, g: 0.118, b: 0.118, a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        // Render rectangles first (background)
        self.rect_renderer.render(&mut render_pass);

        // Render text on top
        // self.text_renderer.render(&self.text_atlas, &mut render_pass).unwrap_or_else(|e| {
        //     tracing::warn!("Text render error: {:?}", e);
        // });
    }

    self.queue.submit(std::iter::once(encoder.finish()));
    surface_texture.present();

    self.frame_timer.end_frame();
    Ok(())
}
```

### Input handling integration:

```rust
fn handle_input(&mut self, event: &winit::event::WindowEvent) {
    use winit::event::WindowEvent;
    match event {
        WindowEvent::MouseInput { state, button, .. } => {
            if *state == winit::event::ElementState::Pressed
                && *button == winit::event::MouseButton::Left
            {
                // Check what was clicked
                if let Some((mx, my)) = self.last_mouse_position {
                    if self.layout.activity_bar.contains(mx, my) {
                        self.activity_bar.handle_click(my, &self.layout.activity_bar);
                        // Toggle sidebar/AI panel based on which item was clicked
                    } else if self.layout.tab_bar.contains(mx, my) {
                        self.tab_bar.handle_click(mx, &self.layout.tab_bar);
                    } else if self.layout.gutter.contains(mx, my) {
                        self.gutter.handle_click(my, &self.layout.gutter);
                    } else if self.layout.scrollbar_v.contains(mx, my) {
                        self.scrollbar.start_drag(my, self.editor.scroll_top());
                    }
                }
            }
        }
        WindowEvent::CursorMoved { position, .. } => {
            let mx = position.x as f32;
            let my = position.y as f32;
            self.last_mouse_position = Some((mx, my));

            if self.layout.activity_bar.contains(mx, my) {
                self.activity_bar.handle_hover(my, &self.layout.activity_bar);
            } else {
                self.activity_bar.hovered_item = None;
            }

            if self.scrollbar.dragging {
                let visible = (self.layout.editor.height / LayoutConstants::LINE_HEIGHT) as usize;
                let new_scroll = self.scrollbar.update_drag(
                    my,
                    &self.layout.scrollbar_v,
                    self.editor.total_lines(),
                    visible,
                );
                self.editor.set_scroll_top(new_scroll);
            }
        }
        WindowEvent::KeyboardInput { event, .. } => {
            // Reset cursor blink on any keypress
            self.cursor_renderer.reset_blink();
            // Existing keyboard handling...
        }
        _ => {}
    }
}
```

### Run `cargo check --package forge-app` — fix ALL errors.
### Run `cargo test --workspace` — fix ALL failures.
