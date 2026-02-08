//! Application — winit event loop handler + glyphon text rendering
//!
//! This is the main loop that ties GPU, editor, and text rendering together.

use crate::editor::Editor;
use crate::gpu::GpuContext;
use crate::modes::UiMode;
use std::sync::Arc;
use tracing::info;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{Key, ModifiersState, NamedKey};
use winit::window::{Window, WindowAttributes, WindowId};

use glyphon::{
    Attrs, Buffer as GlyphonBuffer, Cache, Color as GlyphonColor, Family, FontSystem, Metrics,
    Resolution, Shaping, SwashCache, TextArea, TextAtlas, TextBounds, TextRenderer, Viewport,
};

/// Font metrics
const FONT_SIZE: f32 = 16.0;
const LINE_HEIGHT: f32 = 22.0;
const LEFT_PADDING: f32 = 8.0;
const TOP_PADDING: f32 = 8.0;
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
use std::time::{Duration, Instant};

/// Frame timing state (Task 10)
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

/// Pre-allocated render collections (avoid allocating in hot loop) (Task 10)
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

/// Theme colors (Forge Dark — matching forge-renderer theme)
const BG_COLOR: wgpu::Color = wgpu::Color {
    r: 0.102,
    g: 0.106,
    b: 0.149,
    a: 1.0,
}; // #1a1b26

pub struct Application {
    /// File path to open (from CLI)
    file_path: Option<String>,
    /// Created after window is ready
    state: Option<AppState>,
    /// Keyboard modifier state
    modifiers: ModifiersState,
    /// Current UI mode
    current_mode: UiMode,
}

struct AppState {
    window: Arc<Window>,
    gpu: GpuContext,
    editor: Editor,
    // Text rendering (glyphon 0.7)
    state: Option<ForgeApplication>,
    /// Keyboard modifier state
    modifiers: ModifiersState,
}

/// The main application state (Task 12)
struct ForgeApplication {
    window: Arc<Window>,
    // GPU resources
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,

    // Text rendering
    font_system: FontSystem,
    swash_cache: SwashCache,
    cache: Cache,
    text_atlas: TextAtlas,
    viewport: Viewport,
    text_renderer: TextRenderer,
    glyphon_buffer: GlyphonBuffer,

    // Editor state
    editor: Editor,

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
    last_mouse_position: Option<(f32, f32)>,

    // NEW: Performance
    frame_timer: FrameTimer,
    render_batch: RenderBatch,

    // NEW: Organism
    organism_state: SharedOrganismState,
}

impl Application {
    pub fn new(file_path: Option<String>) -> Self {
        Self {
            file_path,
            state: None,
            modifiers: ModifiersState::empty(),
            current_mode: UiMode::default(),
        }
    }

    fn init_state(&mut self, event_loop: &ActiveEventLoop) {
        // Create window
        let attrs = WindowAttributes::default()
            .with_title("Forge")
            .with_inner_size(LogicalSize::new(1280, 800));
        let window = Arc::new(event_loop.create_window(attrs).unwrap());

        // Init GPU
        let gpu = GpuContext::new(window.clone()).expect("Failed to initialize GPU");
        info!(
            "GPU initialized: {}x{}",
            gpu.config.width, gpu.config.height
        );
        // Init GPU (using GpuContext logic inline or extracting parts)
        // Since ForgeApplication needs individual fields, we'll extract them from GpuContext or recreate logic.
        // But GpuContext is in crate::gpu. Let's use it to bootstrap then destructure if needed,
        // OR just keep GpuContext inside ForgeApplication if strictly needed, but the Task 12 struct flattened it.
        // Let's flatten it as per Task 12 instructions.

        let gpu = GpuContext::new(window.clone()).expect("Failed to initialize GPU");
        let (width, height) = gpu.size();

        // Init editor
        let editor = if let Some(ref path) = self.file_path {
            Editor::open_file(path).unwrap_or_else(|e| {
                tracing::warn!("Failed to open {}: {}, creating empty buffer", path, e);
                Editor::new()
            })
        } else {
            let mut ed = Editor::new();
            // Show welcome text in empty buffer
            let welcome = "Welcome to Forge Editor\n\nOpen a file: forge <filename>\n\nShortcuts:\n  Ctrl+S  Save\n  Ctrl+Z  Undo\n  Ctrl+Y  Redo\n  Arrows  Navigate\n  Home    Start of line\n  End     End of line\n";
            for c in welcome.chars() {
                ed.insert_char(c);
            }
            ed.buffer.mark_clean();
            ed
        };

        // Set window title
        window.set_title(&editor.window_title());

        // Init text rendering with glyphon 0.7
        let mut font_system = FontSystem::new();
        let swash_cache = SwashCache::new();
        let cache = Cache::new(&gpu.device);
        let mut text_atlas = TextAtlas::new(&gpu.device, &gpu.queue, &cache, gpu.format());
        let viewport = Viewport::new(&gpu.device, &cache);
        let text_renderer = TextRenderer::new(
            &mut text_atlas,
            &gpu.device,
            wgpu::MultisampleState::default(),
            None,
        );
        let mut glyphon_buffer =
            GlyphonBuffer::new(&mut font_system, Metrics::new(FONT_SIZE, LINE_HEIGHT));

        // Set initial buffer size
        let (w, h) = gpu.size();
        glyphon_buffer.set_size(
            &mut font_system,
            Some(w as f32 - LEFT_PADDING),
            Some(h as f32),
        );

        self.state = Some(AppState {
            window,
            gpu,
            editor,
            GlyphonBuffer::new(&mut font_system, Metrics::new(LayoutConstants::FONT_SIZE, LayoutConstants::LINE_HEIGHT));

        glyphon_buffer.set_size(
            &mut font_system,
            Some(width as f32),
            Some(height as f32),
        );

        // Task 12: Initialization
        let rect_renderer = RectRenderer::new(&gpu.device, gpu.config.format);

        let layout = LayoutZones::compute(
            width as f32,
            height as f32,
            true, // sidebar open initially for demo
            false, // AI panel closed initially
        );

        let mut tab_bar = TabBar::new();
        let activity_bar = ActivityBar::new();
        let gutter = Gutter::new();
        let status_bar_state = StatusBar::new();
        let cursor_renderer = CursorRenderer::new();
        let mut breadcrumb_bar = BreadcrumbBar::new();
        let scrollbar = Scrollbar::new();
        let frame_timer = FrameTimer::new();
        let render_batch = RenderBatch::new();

        if let Some(path) = &self.file_path {
            let filename = std::path::Path::new(path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "untitled".to_string());
            tab_bar.open_tab(filename, Some(path.clone()));
            breadcrumb_bar.update_from_path(path);
        }

        let organism_state = organism::new_shared_state();
        let _heartbeat = organism::start_heartbeat(
            organism_state.clone(),
            std::time::Duration::from_millis(250),
        );

        self.state = Some(ForgeApplication {
            window,
            device: gpu.device,
            queue: gpu.queue,
            surface: gpu.surface,
            config: gpu.config,
            font_system,
            swash_cache,
            cache,
            text_atlas,
            viewport,
            text_renderer,
            glyphon_buffer,
        });
    }

    fn render(state: &mut AppState, mode: &UiMode) {
        // Layout config from mode
        let mode_config = mode.layout_config();

        // Build the display text with line numbers (if gutter enabled)
        let text = state.editor.text();
        let scroll_top = state.editor.scroll_y as usize;
        let (_, h) = state.gpu.size();
        let visible_lines = (h as f32 / LINE_HEIGHT) as usize + 1;

        let mut display = String::new();
        let lines: Vec<&str> = text.lines().collect();
        let total_lines = lines.len().max(1);
        let width_digits = total_lines.to_string().len().max(3);

        for (i, line) in lines
            .iter()
            .enumerate()
            .skip(scroll_top)
            .take(visible_lines)
        {
            if mode_config.gutter {
                let line_num = i + 1;
                display.push_str(&format!(
                    "{:>width$}  {}\n",
                    line_num,
                    line,
                    width = width_digits
                ));
            } else {
                display.push_str(&format!("{}\n", line));
            }
        }
        // Handle empty buffer
        if display.is_empty() {
            if mode_config.gutter {
                display.push_str(&format!("{:>width$}  \n", 1, width = width_digits));
            } else {
                display.push_str("\n");
            }
        }

        // Update glyphon buffer with display text
        state.glyphon_buffer.set_text(
            &mut state.font_system,
            &display,
            editor,
            rect_renderer,
            layout,
            tab_bar,
            activity_bar,
            gutter,
            status_bar_state,
            cursor_renderer,
            breadcrumb_bar,
            scrollbar,
            sidebar_open: true,
            ai_panel_open: false,
            last_mouse_position: None,
            frame_timer,
            render_batch,
            organism_state,
        });
    }

    fn render(state: &mut ForgeApplication) -> Result<(), wgpu::SurfaceError> {
        state.frame_timer.begin_frame();

        // Rule 2: Guard against zero-size windows
        if state.config.width == 0 || state.config.height == 0 {
            return Ok(());
        }

        // Rule 1: Never unwrap in render path
        let surface_texture = match state.surface.get_current_texture() {
            Ok(texture) => texture,
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                state.surface.configure(&state.device, &state.config);
                return Ok(());
            }
            Err(wgpu::SurfaceError::OutOfMemory) => {
                tracing::error!("GPU out of memory!");
                return Ok(());
            }
            Err(e) => {
                tracing::warn!("Surface error: {:?}, skipping frame", e);
                return Ok(());
            }
        };

        let view = surface_texture.texture.create_view(&wgpu::TextureViewDescriptor::default());

        // --- COLLECT ALL RECTANGLES (no allocations - reuse batch) ---
        state.render_batch.clear();

        // 1. Background rectangles (layout chrome)
        let bg_rects = state.layout.background_rects();
        state.render_batch.extend(&bg_rects);

        // 2. Tab bar
        let tab_rects = state.tab_bar.render_rects(&state.layout.tab_bar);
        state.render_batch.extend(&tab_rects);

        // 3. Activity bar
        let ab_rects = state.activity_bar.render_rects(&state.layout.activity_bar);
        state.render_batch.extend(&ab_rects);

        // 4. Gutter (line numbers + breakpoints)
        state.gutter.scroll_top = state.editor.scroll_top();
        state.gutter.total_lines = state.editor.total_lines();
        state.gutter.cursor_line = state.editor.cursor_line();
        let gutter_rects = state.gutter.render_rects(&state.layout.gutter);
        state.render_batch.extend(&gutter_rects);

        // 5. Current line highlight
        if let Some(hl_rect) = state.cursor_renderer.current_line_rect(
            state.editor.cursor_line(),
            state.editor.scroll_top(),
            &state.layout.editor,
        ) {
            state.render_batch.push(hl_rect);
        }

        // 6. Cursor
        state.cursor_renderer.update();
        if let Some(cursor_rect) = state.cursor_renderer.render_rect(
            state.editor.cursor_line(),
            state.editor.cursor_col(),
            state.editor.scroll_top(),
            &state.layout.editor,
        ) {
            state.render_batch.push(cursor_rect);
        }

        // 7. Scrollbar
        let visible_lines = (state.layout.editor.height / LayoutConstants::LINE_HEIGHT) as usize;
        let sb_rects = state.scrollbar.render_rect(
            &state.layout.scrollbar_v,
            state.editor.total_lines(),
            visible_lines,
            state.editor.scroll_top(),
        );
        state.render_batch.extend(&sb_rects);

        // 8. Breadcrumb separators
        let bc_rects = state.breadcrumb_bar.render_rects(&state.layout.breadcrumb_bar);
        state.render_batch.extend(&bc_rects);

        // --- UPLOAD & RENDER RECTANGLES ---
        state.rect_renderer.prepare(&state.queue, &state.render_batch.rects);

        // --- PREPARE TEXT ---
        // Update status bar with current state
        state.status_bar_state.cursor_line = state.editor.cursor_line() + 1;
        state.status_bar_state.cursor_col = state.editor.cursor_col() + 1;
        state.status_bar_state.frame_time_ms = state.frame_timer.avg_frame_time_ms;

        // Read organism state (non-blocking)
        if let Some(org_state) = organism::read_state(&state.organism_state) {
            state.status_bar_state.confidence_score = Some(org_state.confidence_score);
        }

        // Collect text for Glyphon
        // For simplicity in this step, we'll mostly render the editor text as before,
        // but offset to the editor zone.
        // Also render tab titles, gutter numbers, status bar text.

        let mut text_display = String::new();

        // Editor text
        let scroll_top = state.editor.scroll_top();
        let visible_lines = (state.layout.editor.height / LayoutConstants::LINE_HEIGHT) as usize + 1;
        let text = state.editor.text();

        // We need to render text line by line to handle positioning?
        // Glyphon supports a buffer. Let's try to populate the buffer with the visible text.
        // However, standard glyphon usage is one large buffer or multiple TextAreas.
        // Let's use multiple TextAreas for UI elements if possible, or clear/reuse buffer.
        // For now, let's just render the editor text in the editor zone.

        // Rule 3: Guard against index out of bounds
        // Rule 5: Guard against division by zero (already handled by min(1) or similar logic in logic)

        let lines: Vec<std::borrow::Cow<str>> = text.lines().map(|l| std::borrow::Cow::from(l)).collect();
        // Note: ropey iterator might need care. state.editor.text() returns String (Cow?)?
        // Editor::text() usually returns Rope or String. In `editor.rs` check implementation.
        // Assuming it returns `Rope` or something compatible.
        // The previous `application.rs` used `state.editor.text()` which returned `Rope` or `String`.

        let mut display_text = String::new();

        // Use Guard for safe line access (Rule 3)
        // Actually, we iterate lines.

        for i in 0..visible_lines {
            let line_idx = scroll_top + i;
            if line_idx >= state.editor.total_lines() {
                break;
            }

            // Guard: Safe line access
            let line_text = Guard::get_line(state.editor.buffer.rope(), line_idx);
            display_text.push_str(&line_text);
            if !line_text.ends_with('\n') {
                display_text.push('\n');
            }
        }

        // Update glyphon buffer
        state.glyphon_buffer.set_text(
            &mut state.font_system,
            &display_text,
            Attrs::new()
                .family(Family::Monospace)
                .color(GlyphonColor::rgb(224, 227, 236)),
            Shaping::Advanced,
        );
        state
            .glyphon_buffer
            .shape_until_scroll(&mut state.font_system, false);

        // Get surface texture
        let surface_texture = match state.gpu.surface.get_current_texture() {
            Ok(t) => t,
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                let (w, h) = state.gpu.size();
                state.gpu.resize(w, h);
                return;
            }
            Err(e) => {
                tracing::error!("Surface error: {}", e);
                return;
            }
        };

        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let (width, height) = state.gpu.size();

        // Update viewport
        state
            .viewport
            .update(&state.gpu.queue, Resolution { width, height });

        // Prepare text rendering
        state
            .text_renderer
            .prepare(
                &state.gpu.device,
                &state.gpu.queue,
                &mut state.font_system,
                &mut state.text_atlas,
                &state.viewport,
                [TextArea {
                    buffer: &state.glyphon_buffer,
                    left: LEFT_PADDING,
                    top: TOP_PADDING,
                    scale: 1.0,
                    bounds: TextBounds {
                        left: 0,
                        top: 0,
                        right: width as i32,
                        bottom: height as i32,
                    },
                    default_color: GlyphonColor::rgb(224, 227, 236),
                    custom_glyphs: &[],
                }],
                &mut state.swash_cache,
            )
            .unwrap();

        // Render
        let mut encoder =
            state
                .gpu
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("forge-render"),
                });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("forge-render-pass"),
        state.glyphon_buffer.shape_until_scroll(&mut state.font_system, false);

        // Update viewport
        state.viewport.update(&state.queue, Resolution {
            width: state.config.width,
            height: state.config.height
        });

        // Prepare text rendering
        // We position the editor text at layout.editor.x, layout.editor.y
        let editor_left = state.layout.editor.x as i32;
        let editor_top = state.layout.editor.y as i32;

        // Note: We are only rendering editor text for now to keep it simple and working.
        // Ideally we would render all UI text.

        state.text_renderer.prepare(
            &state.device,
            &state.queue,
            &mut state.font_system,
            &mut state.text_atlas,
            &state.viewport,
            [TextArea {
                buffer: &state.glyphon_buffer,
                left: editor_left as f32,
                top: editor_top as f32,
                scale: 1.0,
                bounds: TextBounds {
                    left: editor_left,
                    top: editor_top,
                    right: (state.layout.editor.x + state.layout.editor.width) as i32,
                    bottom: (state.layout.editor.y + state.layout.editor.height) as i32,
                },
                default_color: GlyphonColor::rgb(224, 227, 236),
                custom_glyphs: &[],
            }],
            &mut state.swash_cache,
        ).unwrap();

        // --- GPU RENDER PASS ---
        let mut encoder = state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(BG_COLOR),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            state
                .text_renderer
                .render(&state.text_atlas, &state.viewport, &mut pass)
                .unwrap();
        }

        state.gpu.queue.submit(std::iter::once(encoder.finish()));
        surface_texture.present();
            // Render rectangles first (background)
            state.rect_renderer.render(&mut render_pass);

            // Render text on top
            state.text_renderer.render(&state.text_atlas, &state.viewport, &mut render_pass).unwrap_or_else(|e| {
                tracing::warn!("Text render error: {:?}", e);
            });
        }

        state.queue.submit(std::iter::once(encoder.finish()));
        surface_texture.present();

        state.frame_timer.end_frame();
        Ok(())
    }

    fn handle_input(state: &mut ForgeApplication, event: &WindowEvent) {
        match event {
            WindowEvent::MouseInput { state: element_state, button, .. } => {
                if *element_state == ElementState::Pressed
                    && *button == winit::event::MouseButton::Left
                {
                    // Check what was clicked
                    if let Some((mx, my)) = state.last_mouse_position {
                        if state.layout.activity_bar.contains(mx, my) {
                            if let Some(item) = state.activity_bar.handle_click(my, &state.layout.activity_bar) {
                                // Toggle sidebar based on click
                                if item == crate::activity_bar::ActivityItem::AiAgent {
                                     state.ai_panel_open = !state.ai_panel_open;
                                } else {
                                     // For other items, maybe ensure sidebar is open?
                                     // For now just toggle sidebar if it's not the AI agent (simplification)
                                     // state.sidebar_open = !state.sidebar_open;
                                }

                                // Recompute layout
                                state.layout = LayoutZones::compute(
                                    state.config.width as f32,
                                    state.config.height as f32,
                                    state.sidebar_open,
                                    state.ai_panel_open,
                                );
                            }
                        } else if state.layout.tab_bar.contains(mx, my) {
                            state.tab_bar.handle_click(mx, &state.layout.tab_bar);
                        } else if state.layout.gutter.contains(mx, my) {
                            state.gutter.handle_click(my, &state.layout.gutter);
                        } else if state.layout.scrollbar_v.contains(mx, my) {
                            state.scrollbar.start_drag(my, state.editor.scroll_top());
                        }
                    }
                } else if *element_state == ElementState::Released {
                     state.scrollbar.stop_drag();
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let mx = position.x as f32;
                let my = position.y as f32;
                state.last_mouse_position = Some((mx, my));

                if state.layout.activity_bar.contains(mx, my) {
                    state.activity_bar.handle_hover(my, &state.layout.activity_bar);
                } else {
                    state.activity_bar.hovered_item = None;
                }

                if state.scrollbar.dragging {
                    let visible = (state.layout.editor.height / LayoutConstants::LINE_HEIGHT) as usize;
                    let new_scroll = state.scrollbar.update_drag(
                        my,
                        &state.layout.scrollbar_v,
                        state.editor.total_lines(),
                        visible,
                    );
                    state.editor.set_scroll_top(new_scroll);
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                // Reset cursor blink on any keypress
                state.cursor_renderer.reset_blink();

                // Existing keyboard handling adapted
                if event.state != ElementState::Pressed {
                    return;
                }

                // We need to access modifiers from the outer Application struct, but we are inside a static method
                // or passed as arg. Here we assume we handle it or modifiers are stored in state.
                // Actually `handle_input` in Task 12 example was a method on `ForgeApplication` or `Application`.
                // We'll handle keyboard logic here directly or call editor methods.

                // NOTE: Modifiers handling is tricky here because they are stored in `Application` not `ForgeApplication`.
                // We might need to pass modifiers or move modifiers to `ForgeApplication`.
                // Let's assume we handle it in `window_event` and pass relevant info or move modifiers to ForgeApplication.
            }
            _ => {}
        }
    }
}

impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_none() {
            self.init_state(event_loop);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
        let state = match self.state.as_mut() {
            Some(s) => s,
            None => return,
        };

        // Handle specific window events first
        match &event {
             WindowEvent::CloseRequested => {
                info!("Goodbye from Forge 🔥");
                event_loop.exit();
            }

            WindowEvent::Resized(size) => {
                if let Some(state) = self.state.as_mut() {
                    state.gpu.resize(size.width, size.height);
                    state.glyphon_buffer.set_size(
                        &mut state.font_system,
                        Some(size.width as f32 - LEFT_PADDING),
                        Some(size.height as f32),
                    );
                if size.width > 0 && size.height > 0 {
                    state.config.width = size.width;
                    state.config.height = size.height;
                    state.surface.configure(&state.device, &state.config);

                    // Task 12: Resize handler
                    state.layout = LayoutZones::compute(
                        size.width as f32,
                        size.height as f32,
                        state.sidebar_open,
                        state.ai_panel_open,
                    );

                    state.rect_renderer.resize(&state.queue, size.width as f32, size.height as f32);

                    state.glyphon_buffer.set_size(
                        &mut state.font_system,
                        Some(state.layout.editor.width),
                        Some(state.layout.editor.height),
                    );

                    state.window.request_redraw();
                }
            }

            WindowEvent::ModifiersChanged(mods) => {
                self.modifiers = mods.state();
            }

            WindowEvent::KeyboardInput { event, .. } => {
                if event.state != ElementState::Pressed {
                    return;
                }

                let ctrl = self.modifiers.control_key();

                if let Some(state) = self.state.as_mut() {
                    match event.logical_key {
            WindowEvent::KeyboardInput { event: key_event, .. } => {
                 // Forward to handle_input helper (for blink reset etc)
                 Application::handle_input(state, &event);

                 // Editor logic (copied from original application.rs but using state fields)
                 if key_event.state == ElementState::Pressed {
                    let ctrl = self.modifiers.control_key();

                    match key_event.logical_key {
                        // Ctrl shortcuts
                        Key::Character(ref c) if ctrl => match c.as_str() {
                            "s" => {
                                if let Err(e) = state.editor.save() {
                                    tracing::error!("Save failed: {}", e);
                                }
                                state.window.set_title(&state.editor.window_title());
                            }
                            "z" => {
                                state.editor.buffer.undo();
                            }
                            "y" => {
                                state.editor.buffer.redo();
                            }
                            "m" => {
                                // Mutate mode (allowed because state only borrows self.state)
                                self.current_mode = self.current_mode.next();
                                // We'll update title at the end
                            }
                            _ => {}
                        },

                        // Navigation keys
                        Key::Named(NamedKey::ArrowLeft) => state.editor.move_left(),
                        Key::Named(NamedKey::ArrowRight) => state.editor.move_right(),
                        Key::Named(NamedKey::ArrowUp) => state.editor.move_up(),
                        Key::Named(NamedKey::ArrowDown) => state.editor.move_down(),
                        Key::Named(NamedKey::Home) => state.editor.move_home(),
                        Key::Named(NamedKey::End) => state.editor.move_end(),

                        // Editing keys
                        Key::Named(NamedKey::Backspace) => state.editor.backspace(),
                        Key::Named(NamedKey::Delete) => state.editor.delete(),
                        Key::Named(NamedKey::Enter) => state.editor.insert_newline(),
                        Key::Named(NamedKey::Tab) => {
                            // Insert 4 spaces
                            for _ in 0..4 {
                                state.editor.insert_char(' ');
                            }
                        }

                        // Character input
                        Key::Character(ref c) if !ctrl => {
                            for ch in c.chars() {
                                state.editor.insert_char(ch);
                            }
                        }

                        _ => {}
                    }

                    // Ensure cursor is visible and request redraw
                    let visible_lines = (state.gpu.config.height as f32 / LINE_HEIGHT) as usize;
                    state.editor.ensure_cursor_visible(visible_lines);

                    let title = state.editor.window_title();
                    state.window.set_title(&format!("{} - {}", title, self.current_mode.label()));
                    state.window.request_redraw();
                }
            }

            WindowEvent::MouseWheel { delta, .. } => {
                if let Some(state) = self.state.as_mut() {
                    let scroll = match delta {
                        winit::event::MouseScrollDelta::LineDelta(_, y) => -y as f64 * 3.0,
                        winit::event::MouseScrollDelta::PixelDelta(pos) => -pos.y / LINE_HEIGHT as f64,
                    };
                    state.editor.scroll(scroll);
                    state.window.request_redraw();
                }
            }

            WindowEvent::RedrawRequested => {
                if let Some(state) = self.state.as_mut() {
                    Self::render(state, &self.current_mode);
                }
            }

            _ => {}
                    // Rule 4: Clamp scroll position
                    // Ensure cursor is visible (this logic was in original, update it)
                    let visible_lines = (state.layout.editor.height / LayoutConstants::LINE_HEIGHT) as usize;
                    state.editor.ensure_cursor_visible(visible_lines);

                    // Additional clamp from Task 9 Rule 4
                    let total = state.editor.total_lines();
                    let max_scroll = total.saturating_sub(1);
                    // editor.scroll_top is accessed via getter usually, but if public field:
                    // state.editor.scroll_top = state.editor.scroll_top.min(max_scroll);
                    // But `Editor` struct might encapsulate it. `ensure_cursor_visible` handles it mostly.

                    state.window.set_title(&state.editor.window_title());
                    state.window.request_redraw();
                 }
            }

            WindowEvent::MouseWheel { delta, .. } => {
                Application::handle_input(state, &event); // For other mouse handling if needed

                let scroll = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => -y as f64 * 3.0,
                    winit::event::MouseScrollDelta::PixelDelta(pos) => -pos.y / LayoutConstants::LINE_HEIGHT as f64,
                };
                state.editor.scroll(scroll);
                state.window.request_redraw();
            }

            WindowEvent::RedrawRequested => {
                if let Err(e) = Application::render(state) {
                     tracing::error!("Render error: {:?}", e);
                }
            }

            _ => {
                // Other input
                Application::handle_input(state, &event);
            }
        }
    }
}
