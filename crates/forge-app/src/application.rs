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
    font_system: FontSystem,
    swash_cache: SwashCache,
    cache: Cache,
    text_atlas: TextAtlas,
    viewport: Viewport,
    text_renderer: TextRenderer,
    glyphon_buffer: GlyphonBuffer,
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
        }
    }
}
