# JULES IMPLEMENTATION — PART 1 OF 4
# Tasks 1-4: Rectangle Renderer, UI Layout, Tab Bar, Activity Bar

> **CRITICAL**: Read ALL of this before writing any code. Complete each task in order.
> After every file change, run `cargo check --package forge-app` and fix ALL errors before proceeding.
> Use `wgpu = "23"` — NOT 24. glyphon 0.7 requires wgpu 23.

---

## TASK 1: Rectangle Renderer (GPU Quad Pipeline)

### Create file: `crates/forge-app/src/rect_renderer.rs`

```rust
use wgpu::util::DeviceExt;

/// A colored rectangle to render on screen
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RectVertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

impl RectVertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
        0 => Float32x2,
        1 => Float32x4,
    ];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<RectVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

/// Rectangle definition before converting to vertices
#[derive(Clone, Debug)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub color: [f32; 4],
}

/// GPU pipeline for rendering colored rectangles
pub struct RectRenderer {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    /// Pre-allocated CPU-side vertex data
    vertices: Vec<RectVertex>,
    /// Pre-allocated CPU-side index data
    indices: Vec<u32>,
    /// Number of indices to draw this frame
    num_indices: u32,
    /// Maximum number of rectangles (pre-allocated)
    max_rects: usize,
    /// Uniform buffer for screen dimensions
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    screen_size: [f32; 2],
    _padding: [f32; 2],
}

impl RectRenderer {
    pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> Self {
        let max_rects = 1024;

        // Create shader module
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Rect Shader"),
            source: wgpu::ShaderSource::Wgsl(RECT_SHADER.into()),
        });

        // Create uniform buffer
        let uniforms = Uniforms {
            screen_size: [1920.0, 1080.0],
            _padding: [0.0; 2],
        };
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Rect Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Rect Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Rect Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Rect Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Rect Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[RectVertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // Pre-allocate buffers
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Rect Vertex Buffer"),
            size: (max_rects * 4 * std::mem::size_of::<RectVertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Rect Index Buffer"),
            size: (max_rects * 6 * std::mem::size_of::<u32>()) as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            vertex_buffer,
            index_buffer,
            vertices: Vec::with_capacity(max_rects * 4),
            indices: Vec::with_capacity(max_rects * 6),
            num_indices: 0,
            max_rects,
            uniform_buffer,
            uniform_bind_group,
        }
    }

    /// Update screen size uniform (call on resize)
    pub fn resize(&self, queue: &wgpu::Queue, width: f32, height: f32) {
        let uniforms = Uniforms {
            screen_size: [width, height],
            _padding: [0.0; 2],
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
    }

    /// Prepare rectangles for rendering (call once per frame before render)
    pub fn prepare(&mut self, queue: &wgpu::Queue, rects: &[Rect]) {
        self.vertices.clear();
        self.indices.clear();

        for (i, rect) in rects.iter().enumerate().take(self.max_rects) {
            let base = (i * 4) as u32;

            // Four corners: top-left, top-right, bottom-right, bottom-left
            self.vertices.push(RectVertex {
                position: [rect.x, rect.y],
                color: rect.color,
            });
            self.vertices.push(RectVertex {
                position: [rect.x + rect.width, rect.y],
                color: rect.color,
            });
            self.vertices.push(RectVertex {
                position: [rect.x + rect.width, rect.y + rect.height],
                color: rect.color,
            });
            self.vertices.push(RectVertex {
                position: [rect.x, rect.y + rect.height],
                color: rect.color,
            });

            // Two triangles per rect
            self.indices.push(base);
            self.indices.push(base + 1);
            self.indices.push(base + 2);
            self.indices.push(base);
            self.indices.push(base + 2);
            self.indices.push(base + 3);
        }

        self.num_indices = self.indices.len() as u32;

        if !self.vertices.is_empty() {
            queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&self.vertices));
            queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&self.indices));
        }
    }

    /// Render all prepared rectangles
    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        if self.num_indices == 0 {
            return;
        }
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
    }
}

const RECT_SHADER: &str = r#"
struct Uniforms {
    screen_size: vec2<f32>,
    _padding: vec2<f32>,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    // Convert pixel coordinates to NDC (-1 to 1)
    let x = (in.position.x / uniforms.screen_size.x) * 2.0 - 1.0;
    let y = 1.0 - (in.position.y / uniforms.screen_size.y) * 2.0;
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
"#;
```

### Update `crates/forge-app/Cargo.toml`

Add these dependencies:

```toml
[dependencies]
# ... existing deps ...
bytemuck = { version = "1", features = ["derive"] }
```

### Update `crates/forge-app/src/main.rs` or `lib.rs`

Add the module:

```rust
mod rect_renderer;
```

### Run `cargo check --package forge-app` — fix ALL errors before proceeding.

---

## TASK 2: VS Code UI Layout System

### Create file: `crates/forge-app/src/ui.rs`

```rust
use crate::rect_renderer::Rect;

/// VS Code color scheme — dark theme
pub mod colors {
    /// Activity bar background (#333333)
    pub const ACTIVITY_BAR: [f32; 4] = [0.2, 0.2, 0.2, 1.0];
    /// Tab bar background (#252526)
    pub const TAB_BAR: [f32; 4] = [0.145, 0.145, 0.149, 1.0];
    /// Active tab background (#1e1e1e)
    pub const TAB_ACTIVE: [f32; 4] = [0.118, 0.118, 0.118, 1.0];
    /// Inactive tab background (#2d2d2d)
    pub const TAB_INACTIVE: [f32; 4] = [0.176, 0.176, 0.176, 1.0];
    /// Breadcrumb bar background (#1e1e1e)
    pub const BREADCRUMB: [f32; 4] = [0.118, 0.118, 0.118, 1.0];
    /// Editor background (#1e1e1e)
    pub const EDITOR_BG: [f32; 4] = [0.118, 0.118, 0.118, 1.0];
    /// Gutter background (#1e1e1e)
    pub const GUTTER: [f32; 4] = [0.118, 0.118, 0.118, 1.0];
    /// Status bar background (#007acc)
    pub const STATUS_BAR: [f32; 4] = [0.0, 0.478, 0.8, 1.0];
    /// Current line highlight (#2a2d2e)
    pub const CURRENT_LINE: [f32; 4] = [0.165, 0.176, 0.18, 1.0];
    /// Sidebar background (#252526)
    pub const SIDEBAR: [f32; 4] = [0.145, 0.145, 0.149, 1.0];
    /// Scrollbar (#424242, semi-transparent)
    pub const SCROLLBAR: [f32; 4] = [0.259, 0.259, 0.259, 0.5];
    /// Separator lines (#404040)
    pub const SEPARATOR: [f32; 4] = [0.251, 0.251, 0.251, 1.0];
    /// AI panel background (#1e1e1e)
    pub const AI_PANEL: [f32; 4] = [0.118, 0.118, 0.118, 1.0];
    /// Text foreground (#cccccc)
    pub const TEXT_FG: [f32; 4] = [0.8, 0.8, 0.8, 1.0];
    /// Dimmed text (#858585)
    pub const TEXT_DIM: [f32; 4] = [0.522, 0.522, 0.522, 1.0];
    /// White text
    pub const TEXT_WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
    /// Cursor color (#aeafad)
    pub const CURSOR: [f32; 4] = [0.682, 0.686, 0.678, 1.0];
    /// Selection color (#264f78)
    pub const SELECTION: [f32; 4] = [0.149, 0.31, 0.471, 0.5];
    /// Error red
    pub const ERROR: [f32; 4] = [0.937, 0.325, 0.314, 1.0];
    /// Warning yellow
    pub const WARNING: [f32; 4] = [0.804, 0.682, 0.263, 1.0];
    /// Success green
    pub const SUCCESS: [f32; 4] = [0.345, 0.663, 0.369, 1.0];
}

/// Pixel dimensions for each UI zone
pub struct LayoutConstants;

impl LayoutConstants {
    pub const ACTIVITY_BAR_WIDTH: f32 = 48.0;
    pub const TAB_BAR_HEIGHT: f32 = 35.0;
    pub const BREADCRUMB_HEIGHT: f32 = 22.0;
    pub const STATUS_BAR_HEIGHT: f32 = 22.0;
    pub const GUTTER_WIDTH: f32 = 60.0;
    pub const SIDEBAR_WIDTH: f32 = 240.0;
    pub const SCROLLBAR_WIDTH: f32 = 14.0;
    pub const TAB_WIDTH: f32 = 160.0;
    pub const TAB_CLOSE_SIZE: f32 = 16.0;
    pub const AI_PANEL_WIDTH: f32 = 400.0;
    pub const SEPARATOR_SIZE: f32 = 1.0;
    pub const LINE_HEIGHT: f32 = 20.0;
    pub const CHAR_WIDTH: f32 = 8.4;  // Approximate for monospace at 14px
    pub const FONT_SIZE: f32 = 14.0;
    pub const SMALL_FONT_SIZE: f32 = 12.0;
}

/// Computed layout zones (recalculated on resize)
#[derive(Clone, Debug)]
pub struct LayoutZones {
    pub window_width: f32,
    pub window_height: f32,
    pub activity_bar: Zone,
    pub sidebar: Option<Zone>,
    pub tab_bar: Zone,
    pub breadcrumb_bar: Zone,
    pub gutter: Zone,
    pub editor: Zone,
    pub status_bar: Zone,
    pub ai_panel: Option<Zone>,
    pub scrollbar_v: Zone,
}

#[derive(Clone, Debug, Default)]
pub struct Zone {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Zone {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px < self.x + self.width && py >= self.y && py < self.y + self.height
    }

    pub fn to_rect(&self, color: [f32; 4]) -> Rect {
        Rect {
            x: self.x,
            y: self.y,
            width: self.width,
            height: self.height,
            color,
        }
    }
}

impl LayoutZones {
    /// Recalculate all zones based on window size and panel visibility
    pub fn compute(
        window_width: f32,
        window_height: f32,
        sidebar_open: bool,
        ai_panel_open: bool,
    ) -> Self {
        let activity_x = 0.0;
        let activity_w = LayoutConstants::ACTIVITY_BAR_WIDTH;

        let sidebar = if sidebar_open {
            Some(Zone::new(
                activity_w,
                0.0,
                LayoutConstants::SIDEBAR_WIDTH,
                window_height - LayoutConstants::STATUS_BAR_HEIGHT,
            ))
        } else {
            None
        };

        let content_x = activity_w + if sidebar_open { LayoutConstants::SIDEBAR_WIDTH } else { 0.0 };
        let ai_panel_w = if ai_panel_open { LayoutConstants::AI_PANEL_WIDTH } else { 0.0 };
        let content_w = (window_width - content_x - ai_panel_w).max(100.0);

        let tab_y = 0.0;
        let breadcrumb_y = LayoutConstants::TAB_BAR_HEIGHT;
        let editor_y = breadcrumb_y + LayoutConstants::BREADCRUMB_HEIGHT;
        let editor_h = window_height - editor_y - LayoutConstants::STATUS_BAR_HEIGHT;
        let status_y = window_height - LayoutConstants::STATUS_BAR_HEIGHT;

        let gutter_w = LayoutConstants::GUTTER_WIDTH;
        let scrollbar_w = LayoutConstants::SCROLLBAR_WIDTH;
        let editor_text_w = (content_w - gutter_w - scrollbar_w).max(50.0);

        let ai_panel = if ai_panel_open {
            Some(Zone::new(
                content_x + content_w,
                tab_y,
                ai_panel_w,
                window_height - LayoutConstants::STATUS_BAR_HEIGHT,
            ))
        } else {
            None
        };

        Self {
            window_width,
            window_height,
            activity_bar: Zone::new(activity_x, 0.0, activity_w, window_height - LayoutConstants::STATUS_BAR_HEIGHT),
            sidebar,
            tab_bar: Zone::new(content_x, tab_y, content_w, LayoutConstants::TAB_BAR_HEIGHT),
            breadcrumb_bar: Zone::new(content_x, breadcrumb_y, content_w, LayoutConstants::BREADCRUMB_HEIGHT),
            gutter: Zone::new(content_x, editor_y, gutter_w, editor_h),
            editor: Zone::new(content_x + gutter_w, editor_y, editor_text_w, editor_h),
            status_bar: Zone::new(0.0, status_y, window_width, LayoutConstants::STATUS_BAR_HEIGHT),
            ai_panel,
            scrollbar_v: Zone::new(content_x + gutter_w + editor_text_w, editor_y, scrollbar_w, editor_h),
        }
    }

    /// Generate all background rectangles for the UI chrome
    pub fn background_rects(&self) -> Vec<Rect> {
        let mut rects = Vec::with_capacity(16);

        // Activity bar
        rects.push(self.activity_bar.to_rect(colors::ACTIVITY_BAR));

        // Sidebar (if open)
        if let Some(ref sb) = self.sidebar {
            rects.push(sb.to_rect(colors::SIDEBAR));
            // Separator between sidebar and editor
            rects.push(Rect {
                x: sb.x + sb.width,
                y: 0.0,
                width: LayoutConstants::SEPARATOR_SIZE,
                height: self.window_height - LayoutConstants::STATUS_BAR_HEIGHT,
                color: colors::SEPARATOR,
            });
        }

        // Tab bar
        rects.push(self.tab_bar.to_rect(colors::TAB_BAR));

        // Breadcrumb bar
        rects.push(self.breadcrumb_bar.to_rect(colors::BREADCRUMB));

        // Gutter
        rects.push(self.gutter.to_rect(colors::GUTTER));

        // Editor background
        rects.push(self.editor.to_rect(colors::EDITOR_BG));

        // Scrollbar track
        rects.push(self.scrollbar_v.to_rect(colors::EDITOR_BG));

        // Status bar
        rects.push(self.status_bar.to_rect(colors::STATUS_BAR));

        // AI Panel (if open)
        if let Some(ref ai) = self.ai_panel {
            rects.push(ai.to_rect(colors::AI_PANEL));
            // Separator between editor and AI panel
            rects.push(Rect {
                x: ai.x - LayoutConstants::SEPARATOR_SIZE,
                y: 0.0,
                width: LayoutConstants::SEPARATOR_SIZE,
                height: self.window_height - LayoutConstants::STATUS_BAR_HEIGHT,
                color: colors::SEPARATOR,
            });
        }

        // Separator between tab bar and breadcrumbs
        rects.push(Rect {
            x: self.tab_bar.x,
            y: self.tab_bar.y + self.tab_bar.height,
            width: self.tab_bar.width,
            height: LayoutConstants::SEPARATOR_SIZE,
            color: colors::SEPARATOR,
        });

        rects
    }
}
```

### Update `crates/forge-app/src/main.rs` or module declarations

```rust
mod ui;
```

### Run `cargo check --package forge-app` — fix ALL errors.

---

## TASK 3: Tab Bar Rendering

### Create file: `crates/forge-app/src/tab_bar.rs`

```rust
use crate::rect_renderer::Rect;
use crate::ui::{colors, LayoutConstants, Zone};

/// Represents a single editor tab
#[derive(Clone, Debug)]
pub struct Tab {
    pub title: String,
    pub file_path: Option<String>,
    pub is_modified: bool,
    pub is_active: bool,
}

/// Tab bar state and rendering
pub struct TabBar {
    pub tabs: Vec<Tab>,
    pub active_index: usize,
    pub scroll_offset: f32,
}

impl TabBar {
    pub fn new() -> Self {
        Self {
            tabs: vec![Tab {
                title: String::from("Welcome"),
                file_path: None,
                is_modified: false,
                is_active: true,
            }],
            active_index: 0,
            scroll_offset: 0.0,
        }
    }

    /// Open a new tab (or activate existing one for same file)
    pub fn open_tab(&mut self, title: String, file_path: Option<String>) {
        // Check if tab for this file already exists
        if let Some(path) = &file_path {
            if let Some(idx) = self.tabs.iter().position(|t| t.file_path.as_ref() == Some(path)) {
                self.set_active(idx);
                return;
            }
        }

        self.tabs.push(Tab {
            title,
            file_path,
            is_modified: false,
            is_active: false,
        });
        self.set_active(self.tabs.len() - 1);
    }

    /// Close a tab by index
    pub fn close_tab(&mut self, index: usize) {
        if self.tabs.len() <= 1 {
            return; // Don't close last tab
        }
        self.tabs.remove(index);
        if self.active_index >= self.tabs.len() {
            self.active_index = self.tabs.len() - 1;
        }
        self.tabs[self.active_index].is_active = true;
    }

    /// Set the active tab
    pub fn set_active(&mut self, index: usize) {
        if index < self.tabs.len() {
            for tab in &mut self.tabs {
                tab.is_active = false;
            }
            self.active_index = index;
            self.tabs[index].is_active = true;
        }
    }

    /// Mark a tab as modified (unsaved)
    pub fn set_modified(&mut self, index: usize, modified: bool) {
        if index < self.tabs.len() {
            self.tabs[index].is_modified = modified;
        }
    }

    /// Generate rectangles for the tab bar
    pub fn render_rects(&self, zone: &Zone) -> Vec<Rect> {
        let mut rects = Vec::with_capacity(self.tabs.len() * 2);
        let tab_width = LayoutConstants::TAB_WIDTH;
        let tab_height = zone.height;

        for (i, tab) in self.tabs.iter().enumerate() {
            let x = zone.x + (i as f32 * tab_width) - self.scroll_offset;

            // Skip tabs scrolled out of view
            if x + tab_width < zone.x || x > zone.x + zone.width {
                continue;
            }

            let bg_color = if tab.is_active {
                colors::TAB_ACTIVE
            } else {
                colors::TAB_INACTIVE
            };

            // Tab background
            rects.push(Rect {
                x,
                y: zone.y,
                width: tab_width,
                height: tab_height,
                color: bg_color,
            });

            // Active tab indicator (blue line on top)
            if tab.is_active {
                rects.push(Rect {
                    x,
                    y: zone.y,
                    width: tab_width,
                    height: 2.0,
                    color: colors::STATUS_BAR, // Blue accent
                });
            }

            // Separator between tabs
            if i > 0 {
                rects.push(Rect {
                    x,
                    y: zone.y + 4.0,
                    width: 1.0,
                    height: tab_height - 8.0,
                    color: colors::SEPARATOR,
                });
            }
        }

        rects
    }

    /// Get tab titles for text rendering (returns (text, x, y, is_active, is_modified) tuples)
    pub fn text_positions(&self, zone: &Zone) -> Vec<(String, f32, f32, bool, bool)> {
        let mut result = Vec::with_capacity(self.tabs.len());
        let tab_width = LayoutConstants::TAB_WIDTH;
        let text_y = zone.y + (zone.height - LayoutConstants::SMALL_FONT_SIZE) / 2.0;

        for (i, tab) in self.tabs.iter().enumerate() {
            let x = zone.x + (i as f32 * tab_width) - self.scroll_offset + 12.0;
            let title = if tab.is_modified {
                format!("● {}", tab.title)
            } else {
                tab.title.clone()
            };
            result.push((title, x, text_y, tab.is_active, tab.is_modified));
        }

        result
    }

    /// Handle click in tab bar zone, returns which tab was clicked (if any)
    pub fn handle_click(&mut self, click_x: f32, zone: &Zone) -> Option<usize> {
        let tab_width = LayoutConstants::TAB_WIDTH;
        let relative_x = click_x - zone.x + self.scroll_offset;
        if relative_x < 0.0 {
            return None;
        }
        let index = (relative_x / tab_width) as usize;
        if index < self.tabs.len() {
            self.set_active(index);
            Some(index)
        } else {
            None
        }
    }
}

impl Default for TabBar {
    fn default() -> Self {
        Self::new()
    }
}
```

### Update module declarations

```rust
mod tab_bar;
```

### Run `cargo check --package forge-app` — fix ALL errors.

---

## TASK 4: Activity Bar (Left Icon Strip)

### Create file: `crates/forge-app/src/activity_bar.rs`

```rust
use crate::rect_renderer::Rect;
use crate::ui::{colors, LayoutConstants, Zone};

/// Activity bar button identifiers
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ActivityItem {
    Explorer,
    Search,
    SourceControl,
    Debug,
    Extensions,
    AiAgent,
    Settings,
}

impl ActivityItem {
    /// Unicode icon character for each item
    pub fn icon_char(&self) -> &'static str {
        match self {
            Self::Explorer => "📁",
            Self::Search => "🔍",
            Self::SourceControl => "⎇",
            Self::Debug => "🐛",
            Self::Extensions => "🧩",
            Self::AiAgent => "🤖",
            Self::Settings => "⚙",
        }
    }

    /// Label for tooltip
    pub fn label(&self) -> &'static str {
        match self {
            Self::Explorer => "Explorer",
            Self::Search => "Search",
            Self::SourceControl => "Source Control",
            Self::Debug => "Debug",
            Self::Extensions => "Extensions",
            Self::AiAgent => "AI Agent",
            Self::Settings => "Settings",
        }
    }

    /// All top items (shown at top of activity bar)
    pub fn top_items() -> &'static [ActivityItem] {
        &[
            ActivityItem::Explorer,
            ActivityItem::Search,
            ActivityItem::SourceControl,
            ActivityItem::Debug,
            ActivityItem::Extensions,
            ActivityItem::AiAgent,
        ]
    }

    /// Bottom items (shown at bottom of activity bar)
    pub fn bottom_items() -> &'static [ActivityItem] {
        &[ActivityItem::Settings]
    }
}

/// Activity bar state
pub struct ActivityBar {
    pub active_item: Option<ActivityItem>,
    pub hovered_item: Option<ActivityItem>,
}

impl ActivityBar {
    pub fn new() -> Self {
        Self {
            active_item: Some(ActivityItem::Explorer),
            hovered_item: None,
        }
    }

    /// Generate rectangles for the activity bar icons
    pub fn render_rects(&self, zone: &Zone) -> Vec<Rect> {
        let mut rects = Vec::with_capacity(16);
        let item_size = LayoutConstants::ACTIVITY_BAR_WIDTH;
        let icon_padding = 4.0;

        // Top items
        for (i, item) in ActivityItem::top_items().iter().enumerate() {
            let y = zone.y + (i as f32 * item_size);

            // Highlight active item
            if self.active_item == Some(*item) {
                // Active indicator (white bar on left)
                rects.push(Rect {
                    x: zone.x,
                    y,
                    width: 2.0,
                    height: item_size,
                    color: colors::TEXT_WHITE,
                });
                // Active background
                rects.push(Rect {
                    x: zone.x + 2.0,
                    y,
                    width: item_size - 2.0,
                    height: item_size,
                    color: [0.25, 0.25, 0.25, 1.0],
                });
            }

            // Hover highlight
            if self.hovered_item == Some(*item) && self.active_item != Some(*item) {
                rects.push(Rect {
                    x: zone.x,
                    y,
                    width: item_size,
                    height: item_size,
                    color: [0.22, 0.22, 0.22, 1.0],
                });
            }
        }

        // Bottom items
        let bottom_items = ActivityItem::bottom_items();
        for (i, item) in bottom_items.iter().enumerate() {
            let y = zone.y + zone.height - ((bottom_items.len() - i) as f32 * item_size);

            if self.active_item == Some(*item) {
                rects.push(Rect {
                    x: zone.x,
                    y,
                    width: 2.0,
                    height: item_size,
                    color: colors::TEXT_WHITE,
                });
            }

            if self.hovered_item == Some(*item) {
                rects.push(Rect {
                    x: zone.x,
                    y,
                    width: item_size,
                    height: item_size,
                    color: [0.22, 0.22, 0.22, 1.0],
                });
            }
        }

        rects
    }

    /// Get icon text positions for rendering
    pub fn text_positions(&self, zone: &Zone) -> Vec<(&'static str, f32, f32, bool)> {
        let mut result = Vec::with_capacity(8);
        let item_size = LayoutConstants::ACTIVITY_BAR_WIDTH;

        // Top items
        for (i, item) in ActivityItem::top_items().iter().enumerate() {
            let x = zone.x + item_size / 2.0 - 8.0;
            let y = zone.y + (i as f32 * item_size) + item_size / 2.0 - 8.0;
            let is_active = self.active_item == Some(*item);
            result.push((item.icon_char(), x, y, is_active));
        }

        // Bottom items
        let bottom_items = ActivityItem::bottom_items();
        for (i, item) in bottom_items.iter().enumerate() {
            let x = zone.x + item_size / 2.0 - 8.0;
            let y = zone.y + zone.height - ((bottom_items.len() - i) as f32 * item_size) + item_size / 2.0 - 8.0;
            let is_active = self.active_item == Some(*item);
            result.push((item.icon_char(), x, y, is_active));
        }

        result
    }

    /// Handle click, returns which item was clicked
    pub fn handle_click(&mut self, click_y: f32, zone: &Zone) -> Option<ActivityItem> {
        let item_size = LayoutConstants::ACTIVITY_BAR_WIDTH;

        // Check top items
        for (i, item) in ActivityItem::top_items().iter().enumerate() {
            let y = zone.y + (i as f32 * item_size);
            if click_y >= y && click_y < y + item_size {
                let was_active = self.active_item == Some(*item);
                if was_active {
                    self.active_item = None; // Toggle off
                } else {
                    self.active_item = Some(*item);
                }
                return Some(*item);
            }
        }

        // Check bottom items
        let bottom_items = ActivityItem::bottom_items();
        for (i, item) in bottom_items.iter().enumerate() {
            let y = zone.y + zone.height - ((bottom_items.len() - i) as f32 * item_size);
            if click_y >= y && click_y < y + item_size {
                self.active_item = Some(*item);
                return Some(*item);
            }
        }

        None
    }

    /// Handle mouse move for hover effects
    pub fn handle_hover(&mut self, hover_y: f32, zone: &Zone) {
        let item_size = LayoutConstants::ACTIVITY_BAR_WIDTH;
        self.hovered_item = None;

        // Check top items
        for (i, item) in ActivityItem::top_items().iter().enumerate() {
            let y = zone.y + (i as f32 * item_size);
            if hover_y >= y && hover_y < y + item_size {
                self.hovered_item = Some(*item);
                return;
            }
        }

        // Check bottom items
        let bottom_items = ActivityItem::bottom_items();
        for (i, item) in bottom_items.iter().enumerate() {
            let y = zone.y + zone.height - ((bottom_items.len() - i) as f32 * item_size);
            if hover_y >= y && hover_y < y + item_size {
                self.hovered_item = Some(*item);
                return;
            }
        }
    }
}

impl Default for ActivityBar {
    fn default() -> Self {
        Self::new()
    }
}
```

### Update module declarations

```rust
mod activity_bar;
```

### Run `cargo check --package forge-app` — fix ALL errors.
### Run `cargo test --workspace` — fix ALL failures.
