//! Application — winit event loop handler + glyphon text rendering
//!
//! This is the main loop that ties GPU, editor, and text rendering together.

use crate::activity_bar::ActivityBar;
use crate::breadcrumb::BreadcrumbBar;
use crate::cursor::CursorRenderer;
use crate::editor::Editor;
use crate::gpu::GpuContext;
use crate::guard::Guard;
use crate::gutter::Gutter;
use crate::modes::UiMode;
use crate::organism::{self, SharedOrganismState};
use crate::rect_renderer::RectRenderer;
use crate::scrollbar::Scrollbar;
use crate::status_bar::StatusBar;
use crate::tab_bar::TabBar;
use crate::ui::{LayoutConstants, LayoutZones};

use std::sync::Arc;
use std::time::Instant;
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

// ─── Theme ───
const BG_COLOR: wgpu::Color = wgpu::Color {
    r: 0.102,
    g: 0.106,
    b: 0.149,
    a: 1.0,
};

// ─── Performance ───

pub struct FrameTimer {
    last_frame: Instant,
    frame_times: [f32; 60],
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

pub struct RenderBatch {
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

// ─── Application ───

pub struct Application {
    file_path: Option<String>,
    state: Option<AppState>,
    modifiers: ModifiersState,
    current_mode: UiMode,
}

/// Unified application state
struct AppState {
    window: Arc<Window>,
    gpu: GpuContext,

    // Text rendering (glyphon)
    font_system: FontSystem,
    swash_cache: SwashCache,
    #[allow(dead_code)]
    cache: Cache,
    text_atlas: TextAtlas,
    viewport: Viewport,
    text_renderer: TextRenderer,

    // Text buffers — one per UI region
    editor_buffer: GlyphonBuffer,
    gutter_buffer: GlyphonBuffer,
    tab_buffer: GlyphonBuffer,
    status_buffer: GlyphonBuffer,
    breadcrumb_buffer: GlyphonBuffer,
    sidebar_buffer: GlyphonBuffer,

    // Editor
    editor: Editor,

    // Rectangle renderer
    rect_renderer: RectRenderer,

    // UI components
    layout: LayoutZones,
    tab_bar: TabBar,
    activity_bar: ActivityBar,
    gutter: Gutter,
    #[allow(dead_code)]
    status_bar_state: StatusBar,
    cursor_renderer: CursorRenderer,
    breadcrumb_bar: BreadcrumbBar,
    scrollbar: Scrollbar,

    // UI state
    sidebar_open: bool,
    ai_panel_open: bool,
    last_mouse_position: Option<(f32, f32)>,

    // Performance
    frame_timer: FrameTimer,
    render_batch: RenderBatch,

    // Organism
    #[allow(dead_code)]
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
        let attrs = WindowAttributes::default()
            .with_title("Forge")
            .with_inner_size(LogicalSize::new(1280, 800));
        let window = Arc::new(event_loop.create_window(attrs).unwrap());

        let gpu = GpuContext::new(window.clone()).expect("Failed to initialize GPU");
        let (width, height) = gpu.size();
        info!("GPU initialized: {}x{}", width, height);

        // Init editor
        let editor = if let Some(ref path) = self.file_path {
            Editor::open_file(path).unwrap_or_else(|e| {
                tracing::warn!("Failed to open {}: {}, creating empty buffer", path, e);
                Editor::new()
            })
        } else {
            let mut ed = Editor::new();
            let welcome = "Welcome to Forge Editor\n\nOpen a file: forge <filename>\n\nShortcuts:\n  Ctrl+S  Save\n  Ctrl+Z  Undo\n  Ctrl+Y  Redo\n  Ctrl+M  Cycle UI Mode\n";
            for c in welcome.chars() {
                ed.insert_char(c);
            }
            ed.buffer.mark_clean();
            ed
        };

        window.set_title(&editor.window_title());

        // Init text rendering
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

        // Create text buffers for each UI region
        let editor_buffer = GlyphonBuffer::new(
            &mut font_system,
            Metrics::new(LayoutConstants::FONT_SIZE, LayoutConstants::LINE_HEIGHT),
        );
        let gutter_buffer = GlyphonBuffer::new(
            &mut font_system,
            Metrics::new(
                LayoutConstants::SMALL_FONT_SIZE,
                LayoutConstants::LINE_HEIGHT,
            ),
        );
        let tab_buffer = GlyphonBuffer::new(
            &mut font_system,
            Metrics::new(
                LayoutConstants::SMALL_FONT_SIZE,
                LayoutConstants::LINE_HEIGHT,
            ),
        );
        let status_buffer = GlyphonBuffer::new(
            &mut font_system,
            Metrics::new(
                LayoutConstants::SMALL_FONT_SIZE,
                LayoutConstants::LINE_HEIGHT,
            ),
        );
        let breadcrumb_buffer = GlyphonBuffer::new(
            &mut font_system,
            Metrics::new(
                LayoutConstants::SMALL_FONT_SIZE,
                LayoutConstants::LINE_HEIGHT,
            ),
        );
        let sidebar_buffer = GlyphonBuffer::new(
            &mut font_system,
            Metrics::new(
                LayoutConstants::SMALL_FONT_SIZE,
                LayoutConstants::LINE_HEIGHT,
            ),
        );

        // Init rectangle renderer
        let rect_renderer = RectRenderer::new(&gpu.device, gpu.format());

        // Compute layout
        let layout = LayoutZones::compute(width as f32, height as f32, true, false);

        // Init UI components
        let mut tab_bar = TabBar::new();
        let activity_bar = ActivityBar::new();
        let gutter = Gutter::new();
        let status_bar_state = StatusBar::new();
        let cursor_renderer = CursorRenderer::new();
        let mut breadcrumb_bar = BreadcrumbBar::new();
        let scrollbar = Scrollbar::new();

        if let Some(path) = &self.file_path {
            let filename = std::path::Path::new(path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "untitled".to_string());
            tab_bar.open_tab(filename, Some(path.clone()));
            breadcrumb_bar.update_from_path(path);
        }

        // Organism heartbeat
        let organism_state = organism::new_shared_state();
        let _heartbeat = organism::start_heartbeat(
            organism_state.clone(),
            std::time::Duration::from_millis(250),
        );

        self.state = Some(AppState {
            window,
            gpu,
            font_system,
            swash_cache,
            cache,
            text_atlas,
            viewport,
            text_renderer,
            editor_buffer,
            gutter_buffer,
            tab_buffer,
            status_buffer,
            breadcrumb_buffer,
            sidebar_buffer,
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
            frame_timer: FrameTimer::new(),
            render_batch: RenderBatch::new(),
            organism_state,
        });
    }

    /// Main render function
    fn render(state: &mut AppState, mode: &UiMode) {
        state.frame_timer.begin_frame();

        let (width, height) = state.gpu.size();
        if width == 0 || height == 0 {
            return;
        }

        let mode_config = mode.layout_config();

        // ─── COLLECT RECTANGLES ───
        state.render_batch.clear();

        // Background rectangles
        let bg_rects = state.layout.background_rects();
        state.render_batch.extend(&bg_rects);

        // Tab bar
        if mode_config.tab_bar {
            let tab_rects = state.tab_bar.render_rects(&state.layout.tab_bar);
            state.render_batch.extend(&tab_rects);
        }

        // Activity bar
        if mode_config.activity_bar {
            let ab_rects = state.activity_bar.render_rects(&state.layout.activity_bar);
            state.render_batch.extend(&ab_rects);
        }

        // Gutter
        if mode_config.gutter {
            state.gutter.scroll_top = state.editor.scroll_top();
            state.gutter.total_lines = state.editor.total_lines();
            state.gutter.cursor_line = state.editor.cursor_line();
            let gutter_rects = state.gutter.render_rects(&state.layout.gutter);
            state.render_batch.extend(&gutter_rects);
        }

        // Current line highlight
        if let Some(hl_rect) = state.cursor_renderer.current_line_rect(
            state.editor.cursor_line(),
            state.editor.scroll_top(),
            &state.layout.editor,
        ) {
            state.render_batch.push(hl_rect);
        }

        // Cursor
        if mode_config.cursor_blink {
            state.cursor_renderer.update();
        }
        if let Some(cursor_rect) = state.cursor_renderer.render_rect(
            state.editor.cursor_line(),
            state.editor.cursor_col(),
            state.editor.scroll_top(),
            &state.layout.editor,
        ) {
            state.render_batch.push(cursor_rect);
        }

        // Scrollbar
        let visible_lines = (state.layout.editor.height / LayoutConstants::LINE_HEIGHT) as usize;
        let sb_rects = state.scrollbar.render_rect(
            &state.layout.scrollbar_v,
            state.editor.total_lines(),
            visible_lines,
            state.editor.scroll_top(),
        );
        state.render_batch.extend(&sb_rects);

        // Breadcrumb
        if mode_config.breadcrumbs {
            let bc_rects = state
                .breadcrumb_bar
                .render_rects(&state.layout.breadcrumb_bar);
            state.render_batch.extend(&bc_rects);
        }

        // Upload rectangles
        state
            .rect_renderer
            .prepare(&state.gpu.queue, &state.render_batch.rects);

        // ─── TEXT CONTENT ───

        // 1. Editor text
        let scroll_top = state.editor.scroll_top();
        let vis_lines = (state.layout.editor.height / LayoutConstants::LINE_HEIGHT) as usize + 1;
        let mut editor_text = String::new();
        for i in 0..vis_lines {
            let line_idx = scroll_top + i;
            if line_idx >= state.editor.total_lines() {
                break;
            }
            let line = Guard::get_line(state.editor.buffer.rope(), line_idx);
            editor_text.push_str(&line);
            if !line.ends_with('\n') {
                editor_text.push('\n');
            }
        }

        state.editor_buffer.set_size(
            &mut state.font_system,
            Some(state.layout.editor.width),
            Some(state.layout.editor.height),
        );
        state.editor_buffer.set_text(
            &mut state.font_system,
            &editor_text,
            Attrs::new()
                .family(Family::Monospace)
                .color(GlyphonColor::rgb(212, 212, 212)),
            Shaping::Advanced,
        );
        state
            .editor_buffer
            .shape_until_scroll(&mut state.font_system, false);

        // 2. Gutter (line numbers)
        let mut gutter_text = String::new();
        for i in 0..vis_lines {
            let line_idx = scroll_top + i;
            if line_idx >= state.editor.total_lines() {
                break;
            }
            let line_num = line_idx + 1;
            // Right-align line numbers in gutter
            gutter_text.push_str(&format!("{:>4}\n", line_num));
        }

        state.gutter_buffer.set_size(
            &mut state.font_system,
            Some(state.layout.gutter.width - 8.0),
            Some(state.layout.gutter.height),
        );
        state.gutter_buffer.set_text(
            &mut state.font_system,
            &gutter_text,
            Attrs::new()
                .family(Family::Monospace)
                .color(GlyphonColor::rgb(133, 133, 133)),
            Shaping::Advanced,
        );
        state
            .gutter_buffer
            .shape_until_scroll(&mut state.font_system, false);

        // 3. Tab bar text
        let tab_text = if state.tab_bar.tabs.is_empty() {
            "  Welcome".to_string()
        } else {
            state
                .tab_bar
                .tabs
                .iter()
                .enumerate()
                .map(|(i, tab)| {
                    if i == state.tab_bar.active_index {
                        format!(" ● {} ", tab.title)
                    } else {
                        format!("   {}  ", tab.title)
                    }
                })
                .collect::<Vec<_>>()
                .join("│")
        };

        state.tab_buffer.set_size(
            &mut state.font_system,
            Some(state.layout.tab_bar.width),
            Some(state.layout.tab_bar.height),
        );
        state.tab_buffer.set_text(
            &mut state.font_system,
            &tab_text,
            Attrs::new()
                .family(Family::SansSerif)
                .color(GlyphonColor::rgb(200, 200, 200)),
            Shaping::Advanced,
        );
        state
            .tab_buffer
            .shape_until_scroll(&mut state.font_system, false);

        // 4. Status bar text
        let cursor_line = state.editor.cursor_line() + 1;
        let cursor_col = state.editor.cursor_col() + 1;
        let fps = if state.frame_timer.avg_frame_time_ms > 0.0 {
            (1000.0 / state.frame_timer.avg_frame_time_ms) as u32
        } else {
            0
        };
        let status_text = format!(
            "  Forge IDE   │  Ln {}, Col {}  │  UTF-8  │  Rust  │  {} fps  │  {}  ",
            cursor_line,
            cursor_col,
            fps,
            mode.label()
        );

        state.status_buffer.set_size(
            &mut state.font_system,
            Some(state.layout.status_bar.width),
            Some(state.layout.status_bar.height),
        );
        state.status_buffer.set_text(
            &mut state.font_system,
            &status_text,
            Attrs::new()
                .family(Family::SansSerif)
                .color(GlyphonColor::rgb(255, 255, 255)),
            Shaping::Advanced,
        );
        state
            .status_buffer
            .shape_until_scroll(&mut state.font_system, false);

        // 5. Breadcrumb text
        let breadcrumb_text = if state.breadcrumb_bar.segments.is_empty() {
            "  src > main.rs".to_string()
        } else {
            let path: Vec<&str> = state
                .breadcrumb_bar
                .segments
                .iter()
                .map(|s| s.text.as_str())
                .collect();
            format!("  {}", path.join(" > "))
        };

        state.breadcrumb_buffer.set_size(
            &mut state.font_system,
            Some(state.layout.breadcrumb_bar.width),
            Some(state.layout.breadcrumb_bar.height),
        );
        state.breadcrumb_buffer.set_text(
            &mut state.font_system,
            &breadcrumb_text,
            Attrs::new()
                .family(Family::SansSerif)
                .color(GlyphonColor::rgb(170, 170, 170)),
            Shaping::Advanced,
        );
        state
            .breadcrumb_buffer
            .shape_until_scroll(&mut state.font_system, false);

        // 6. Sidebar text (file explorer placeholder)
        let sidebar_text = if state.sidebar_open {
            "  EXPLORER\n\n  📁 src\n    📄 main.rs\n    📄 editor.rs\n    📄 gpu.rs\n    📄 ui.rs\n  📁 crates\n  📄 Cargo.toml\n  📄 README.md".to_string()
        } else {
            String::new()
        };

        if let Some(ref sb) = state.layout.sidebar {
            state.sidebar_buffer.set_size(
                &mut state.font_system,
                Some(sb.width - 8.0),
                Some(sb.height),
            );
        }
        state.sidebar_buffer.set_text(
            &mut state.font_system,
            &sidebar_text,
            Attrs::new()
                .family(Family::SansSerif)
                .color(GlyphonColor::rgb(200, 200, 200)),
            Shaping::Advanced,
        );
        state
            .sidebar_buffer
            .shape_until_scroll(&mut state.font_system, false);

        // ─── UPDATE VIEWPORT ───
        state
            .viewport
            .update(&state.gpu.queue, Resolution { width, height });

        // ─── PREPARE TEXT AREAS ───
        let ed = &state.layout.editor;
        let gut = &state.layout.gutter;
        let tab = &state.layout.tab_bar;
        let sb = &state.layout.status_bar;
        let bc = &state.layout.breadcrumb_bar;

        let mut text_areas: Vec<TextArea> = vec![
            // Editor text
            TextArea {
                buffer: &state.editor_buffer,
                left: ed.x,
                top: ed.y,
                scale: 1.0,
                bounds: TextBounds {
                    left: ed.x as i32,
                    top: ed.y as i32,
                    right: (ed.x + ed.width) as i32,
                    bottom: (ed.y + ed.height) as i32,
                },
                default_color: GlyphonColor::rgb(212, 212, 212),
                custom_glyphs: &[],
            },
            // Gutter line numbers
            TextArea {
                buffer: &state.gutter_buffer,
                left: gut.x + 4.0,
                top: gut.y,
                scale: 1.0,
                bounds: TextBounds {
                    left: gut.x as i32,
                    top: gut.y as i32,
                    right: (gut.x + gut.width) as i32,
                    bottom: (gut.y + gut.height) as i32,
                },
                default_color: GlyphonColor::rgb(133, 133, 133),
                custom_glyphs: &[],
            },
            // Tab bar
            TextArea {
                buffer: &state.tab_buffer,
                left: tab.x,
                top: tab.y + 9.0, // vertically center in tab bar
                scale: 1.0,
                bounds: TextBounds {
                    left: tab.x as i32,
                    top: tab.y as i32,
                    right: (tab.x + tab.width) as i32,
                    bottom: (tab.y + tab.height) as i32,
                },
                default_color: GlyphonColor::rgb(200, 200, 200),
                custom_glyphs: &[],
            },
            // Status bar
            TextArea {
                buffer: &state.status_buffer,
                left: sb.x,
                top: sb.y + 3.0,
                scale: 1.0,
                bounds: TextBounds {
                    left: sb.x as i32,
                    top: sb.y as i32,
                    right: (sb.x + sb.width) as i32,
                    bottom: (sb.y + sb.height) as i32,
                },
                default_color: GlyphonColor::rgb(255, 255, 255),
                custom_glyphs: &[],
            },
            // Breadcrumb bar
            TextArea {
                buffer: &state.breadcrumb_buffer,
                left: bc.x,
                top: bc.y + 3.0,
                scale: 1.0,
                bounds: TextBounds {
                    left: bc.x as i32,
                    top: bc.y as i32,
                    right: (bc.x + bc.width) as i32,
                    bottom: (bc.y + bc.height) as i32,
                },
                default_color: GlyphonColor::rgb(170, 170, 170),
                custom_glyphs: &[],
            },
        ];

        // Sidebar text area (only if sidebar is open)
        if let Some(ref sidebar) = state.layout.sidebar {
            text_areas.push(TextArea {
                buffer: &state.sidebar_buffer,
                left: sidebar.x + 4.0,
                top: sidebar.y + 8.0,
                scale: 1.0,
                bounds: TextBounds {
                    left: sidebar.x as i32,
                    top: sidebar.y as i32,
                    right: (sidebar.x + sidebar.width) as i32,
                    bottom: (sidebar.y + sidebar.height) as i32,
                },
                default_color: GlyphonColor::rgb(200, 200, 200),
                custom_glyphs: &[],
            });
        }

        if let Err(e) = state.text_renderer.prepare(
            &state.gpu.device,
            &state.gpu.queue,
            &mut state.font_system,
            &mut state.text_atlas,
            &state.viewport,
            text_areas,
            &mut state.swash_cache,
        ) {
            tracing::warn!("Text prepare error: {:?}", e);
        }

        // ─── GPU RENDER PASS ───
        let surface_texture = match state.gpu.surface.get_current_texture() {
            Ok(t) => t,
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                state.gpu.resize(width, height);
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

        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            state
                .gpu
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("forge-render"),
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

            // Rectangles first (background chrome)
            state.rect_renderer.render(&mut render_pass);

            // Text on top
            if let Err(e) =
                state
                    .text_renderer
                    .render(&state.text_atlas, &state.viewport, &mut render_pass)
            {
                tracing::warn!("Text render error: {:?}", e);
            }
        }

        state.gpu.queue.submit(std::iter::once(encoder.finish()));
        surface_texture.present();

        state.frame_timer.end_frame();
    }

    /// Handle mouse/input events on UI components
    fn handle_input(state: &mut AppState, event: &WindowEvent) {
        match event {
            WindowEvent::MouseInput {
                state: element_state,
                button,
                ..
            } => {
                if *element_state == ElementState::Pressed
                    && *button == winit::event::MouseButton::Left
                {
                    if let Some((mx, my)) = state.last_mouse_position {
                        if state.layout.activity_bar.contains(mx, my) {
                            if let Some(item) = state
                                .activity_bar
                                .handle_click(my, &state.layout.activity_bar)
                            {
                                if item == crate::activity_bar::ActivityItem::AiAgent {
                                    state.ai_panel_open = !state.ai_panel_open;
                                }
                                let (w, h) = state.gpu.size();
                                state.layout = LayoutZones::compute(
                                    w as f32,
                                    h as f32,
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
                    state
                        .activity_bar
                        .handle_hover(my, &state.layout.activity_bar);
                } else {
                    state.activity_bar.hovered_item = None;
                }

                if state.scrollbar.dragging {
                    let visible =
                        (state.layout.editor.height / LayoutConstants::LINE_HEIGHT) as usize;
                    let new_scroll = state.scrollbar.update_drag(
                        my,
                        &state.layout.scrollbar_v,
                        state.editor.total_lines(),
                        visible,
                    );
                    state.editor.set_scroll_top(new_scroll);
                }
            }
            _ => {}
        }
    }
}

// ─── winit ApplicationHandler ───

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
        let state = match self.state.as_mut() {
            Some(s) => s,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => {
                info!("Goodbye from Forge 🔥");
                event_loop.exit();
            }

            WindowEvent::Resized(size) => {
                if size.width > 0 && size.height > 0 {
                    state.gpu.resize(size.width, size.height);

                    state.layout = LayoutZones::compute(
                        size.width as f32,
                        size.height as f32,
                        state.sidebar_open,
                        state.ai_panel_open,
                    );

                    state.rect_renderer.resize(
                        &state.gpu.queue,
                        size.width as f32,
                        size.height as f32,
                    );

                    state.window.request_redraw();
                }
            }

            WindowEvent::ModifiersChanged(mods) => {
                self.modifiers = mods.state();
            }

            WindowEvent::KeyboardInput {
                event: ref key_event,
                ..
            } => {
                state.cursor_renderer.reset_blink();

                if key_event.state != ElementState::Pressed {
                    return;
                }

                let ctrl = self.modifiers.control_key();

                match key_event.logical_key {
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
                            self.current_mode = self.current_mode.next();
                        }
                        _ => {}
                    },

                    Key::Named(NamedKey::ArrowLeft) => state.editor.move_left(),
                    Key::Named(NamedKey::ArrowRight) => state.editor.move_right(),
                    Key::Named(NamedKey::ArrowUp) => state.editor.move_up(),
                    Key::Named(NamedKey::ArrowDown) => state.editor.move_down(),
                    Key::Named(NamedKey::Home) => state.editor.move_home(),
                    Key::Named(NamedKey::End) => state.editor.move_end(),

                    Key::Named(NamedKey::Backspace) => state.editor.backspace(),
                    Key::Named(NamedKey::Delete) => state.editor.delete(),
                    Key::Named(NamedKey::Enter) => state.editor.insert_newline(),
                    Key::Named(NamedKey::Tab) => {
                        for _ in 0..4 {
                            state.editor.insert_char(' ');
                        }
                    }

                    Key::Character(ref c) if !ctrl => {
                        for ch in c.chars() {
                            state.editor.insert_char(ch);
                        }
                    }

                    _ => {}
                }

                let visible_lines =
                    (state.layout.editor.height / LayoutConstants::LINE_HEIGHT) as usize;
                state.editor.ensure_cursor_visible(visible_lines);

                let title = state.editor.window_title();
                state
                    .window
                    .set_title(&format!("{} - {}", title, self.current_mode.label()));
                state.window.request_redraw();
            }

            WindowEvent::MouseWheel { delta, .. } => {
                let scroll = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => -y as f64 * 3.0,
                    winit::event::MouseScrollDelta::PixelDelta(pos) => {
                        -pos.y / LayoutConstants::LINE_HEIGHT as f64
                    }
                };
                state.editor.scroll(scroll);
                state.window.request_redraw();
            }

            WindowEvent::RedrawRequested => {
                Application::render(state, &self.current_mode);
            }

            _ => {
                Application::handle_input(state, &event);
            }
        }
    }
}
