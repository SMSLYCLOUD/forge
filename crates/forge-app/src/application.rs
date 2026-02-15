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
use crate::tab_manager::TabManager;
use crate::ui::{LayoutConstants, LayoutZones};

use forge_agent::agent::{Agent, AgentRequest, AgentResponse, AgentStatus, EditorContext};
use forge_lsp::{LspClient, LspServer};
use std::sync::Arc;
use std::time::Instant;
use tokio::runtime::Runtime;
use tracing::info;
use url::Url;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoopProxy};
use winit::keyboard::{Key, ModifiersState, NamedKey};
use winit::window::{Window, WindowAttributes, WindowId};

use forge_syntax::colors::default_color;
use forge_syntax::highlighter::TokenType;

#[path = "application_impl_debug.rs"]
mod application_impl_debug;
#[path = "accessibility.rs"]
mod accessibility;

use glyphon::{
    Attrs, Buffer as GlyphonBuffer, Cache, Color as GlyphonColor, Family, FontSystem, Metrics,
    Resolution, Shaping, SwashCache, TextArea, TextAtlas, TextBounds, TextRenderer, Viewport,
};

// ─── Theme ───
const BG_COLOR: wgpu::Color = wgpu::Color {
    r: 0.051,
    g: 0.043,
    b: 0.102,
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

#[derive(Debug)]
pub enum UserEvent {
    AccessKitEvent(accesskit_winit::Event),
}

impl From<accesskit_winit::Event> for UserEvent {
    fn from(event: accesskit_winit::Event) -> Self {
        UserEvent::AccessKitEvent(event)
    }
}

pub struct Application {
    proxy: EventLoopProxy<UserEvent>,
    file_path: Option<String>,
    screenshot_path: Option<String>,
    debug_zones: bool,
    state: Option<AppState>,
    modifiers: ModifiersState,
    current_mode: UiMode,
    config: forge_config::ForgeConfig,
    theme: forge_theme::Theme,

    find_bar: crate::find_bar::FindBar,
    replace_bar: crate::replace_bar::ReplaceBar,
    go_to_line: crate::go_to_line::GoToLine,
    command_palette: crate::command_palette::CommandPalette,
    bottom_panel: crate::bottom_panel::BottomPanel,
    notifications: crate::notifications::NotificationManager,
    context_menu: crate::context_menu::ContextMenu,
    settings_ui: crate::settings_ui::SettingsUi,
    zen_mode: crate::zen_mode::ZenMode,

    // Async Runtime
    rt: Arc<Runtime>,

    // Phase 2: LSP
    lsp_client: Option<Arc<LspClient>>,

    // Phase 3: AI Agent
    // agent: Option<Arc<forge_agent::agent::AgentRuntime>>,

    // Phase 5: Debug + Plugins
    debug_ui: Option<crate::debug_ui::DebugUi>,
    plugin_runtime: Option<forge_plugin::PluginRuntime>,
    extension_host: Option<forge_extension_host::ExtensionHost>,
    search_panel: crate::search_panel::SearchPanel,

    // Phase 4: Intelligence Layer
    ghost_tabs: forge_anticipation::GhostTabsEngine,
    anomaly_detector: forge_immune::AnomalyDetector,
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
    split_editor_buffer: GlyphonBuffer, // For split view
    gutter_buffer: GlyphonBuffer,
    split_gutter_buffer: GlyphonBuffer, // For split view
    tab_buffer: GlyphonBuffer,
    status_buffer: GlyphonBuffer,
    breadcrumb_buffer: GlyphonBuffer,
    sidebar_buffer: GlyphonBuffer,
    bottom_panel_buffer: GlyphonBuffer,
    activity_bar_buffer: GlyphonBuffer,
    overlay_buffer: GlyphonBuffer,

    // Editor & File Management
    tab_manager: TabManager,
    file_explorer: crate::file_explorer::FileExplorer,

    // Terminal
    terminal: Option<forge_terminal::Terminal>,
    bottom_panel_focused: bool,

    // AI Agent
    agent: Option<Agent>,

    // Debug UI
    debug_ui: crate::debug_ui::DebugUi,
    debug_toolbar: crate::debug_toolbar::DebugToolbar,

    // Rectangle renderer
    rect_renderer: RectRenderer,

    // Debugging
    debug_zones: bool,

    // UI components
    dock_tree: crate::dock::DockTree,
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

    // Sidebar Mode
    sidebar_mode: crate::ui::SidebarMode,

    // Accessibility
    accessibility_manager: self::accessibility::AccessibilityManager,

    // Event Loop Proxy (for async callbacks)
    event_tx: std::sync::mpsc::Sender<AppEvent>,
    event_rx: std::sync::mpsc::Receiver<AppEvent>,
}

#[derive(Debug)]
pub enum AppEvent {
    GoToLocation(lsp_types::Location),
}

impl Application {
    pub fn new(
        proxy: EventLoopProxy<UserEvent>,
        file_path: Option<String>,
        screenshot_path: Option<String>,
        debug_zones: bool,
    ) -> Self {
        let config = forge_config::ForgeConfig::default();
        let theme = forge_theme::Theme::default_dark();

        let find_bar = crate::find_bar::FindBar::default();
        let replace_bar = crate::replace_bar::ReplaceBar::default();
        let go_to_line = crate::go_to_line::GoToLine::default();
        let command_palette = crate::command_palette::CommandPalette::default();
        let bottom_panel = crate::bottom_panel::BottomPanel::default();
        let notifications = crate::notifications::NotificationManager::default();
        let context_menu = crate::context_menu::ContextMenu::default();
        let settings_ui = crate::settings_ui::SettingsUi::new();
        let zen_mode = crate::zen_mode::ZenMode::new();

        // Phase 5: Debug UI (sync init)
        let debug_ui = Some(crate::debug_ui::DebugUi::new());

        let search_panel = crate::search_panel::SearchPanel::new();

        // Phase 5: Plugin runtime (sync init)
        let plugin_runtime = match forge_plugin::PluginRuntime::new() {
            Ok(rt) => {
                info!("Plugin runtime initialized (WASM)");
                Some(rt)
            }
            Err(e) => {
                tracing::warn!(
                    "Plugin runtime init failed: {} — continuing without plugins",
                    e
                );
                None
            }
        };

        // Epic 2: Extension Host
        let extension_host = Some(forge_extension_host::ExtensionHost::new());

        // Phase 4: Intelligence layer (sync init)
        let ghost_tabs = forge_anticipation::GhostTabsEngine::new();
        let anomaly_detector = forge_immune::AnomalyDetector::new(100);

        // Async Runtime
        let rt = Arc::new(Runtime::new().expect("Failed to create Tokio runtime"));

        // LSP Init
        let lsp_client =
            if let Ok(server) = rt.block_on(async { LspServer::spawn("rust-analyzer", &[]) }) {
                info!("LSP: rust-analyzer spawned");
                let mut client = LspClient::new(server);
                if let Err(e) = client.initialize_transport() {
                    tracing::warn!("LSP: Failed to initialize transport: {}", e);
                    None
                } else {
                    let client = Arc::new(client);
                    let c = client.clone();
                    let cwd = std::env::current_dir().unwrap_or_default();
                    if let Ok(cwd_url) = Url::from_directory_path(&cwd) {
                        rt.spawn(async move {
                            if let Err(e) = c.initialize(cwd_url).await {
                                tracing::warn!("LSP: Initialize failed: {}", e);
                            } else {
                                info!("LSP: Initialized");
                            }
                        });
                    }
                    Some(client)
                }
            } else {
                tracing::warn!("LSP: rust-analyzer not found or failed to spawn");
                None
            };

        Self {
            proxy,
            rt,
            lsp_client,
            file_path,
            screenshot_path,
            debug_zones,
            state: None,
            modifiers: ModifiersState::empty(),
            current_mode: UiMode::default(),
            config,
            theme,
            find_bar,
            replace_bar,
            go_to_line,
            command_palette,
            bottom_panel,
            notifications,
            context_menu,
            settings_ui,
            zen_mode,
            debug_ui,
            plugin_runtime,
            extension_host,
            search_panel,
            ghost_tabs,
            anomaly_detector,
        }
    }

    fn init_state(&mut self, event_loop: &ActiveEventLoop) {
        let attrs = WindowAttributes::default()
            .with_title("Forge")
            .with_inner_size(LogicalSize::new(1280, 800));
        let window = Arc::new(event_loop.create_window(attrs).unwrap());

        let accessibility_manager =
            self::accessibility::AccessibilityManager::new(&window, self.proxy.clone());

        let gpu = GpuContext::new(window.clone()).expect("Failed to initialize GPU");
        let (width, height) = gpu.size();
        info!("GPU initialized: {}x{}", width, height);

        // Init tab manager & file explorer
        let mut tab_manager = TabManager::new();
        tab_manager.open_scratch(); // Ensure keyboard input works from launch
        if let Some(ref path) = self.file_path {
            if let Err(e) = tab_manager.open_file(path) {
                tracing::warn!("Failed to open {}: {}", path, e);
            }
            // Phase 4: Record file open in ghost tabs for prediction
            self.ghost_tabs.on_file_open(path);
        }

        let mut file_explorer = crate::file_explorer::FileExplorer::new();
        let cwd = std::env::current_dir().unwrap_or_default();
        // Don't auto-scan at startup — VS Code only shows files when user opens Explorer

        // LSP: Send didOpen if file is opened
        if let Some(ref path) = self.file_path {
            if let Some(client) = &self.lsp_client {
                if let Ok(path_abs) = std::fs::canonicalize(path) {
                    if let Ok(uri) = Url::from_file_path(&path_abs) {
                        let text = std::fs::read_to_string(&path_abs).unwrap_or_default();
                        let client = client.clone();
                        self.rt.spawn(async move {
                            let _ = client.did_open(uri, text, "rust".to_string()).await;
                        });
                    }
                }
            }
        }

        // Phase 3: AI Agent init stub
        // TODO: When tokio runtime is available, spawn agent:
        //   let agent = forge_agent::agent::AgentRuntime::new(config);
        //   agent.start().await;
        info!("AI Agent: will be spawned when async runtime is available");

        let window_title = tab_manager
            .active_editor()
            .map(|e| e.window_title())
            .unwrap_or_else(|| "Forge — [no file]".to_string());
        window.set_title(&window_title);

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
        let split_editor_buffer = GlyphonBuffer::new(
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
        let split_gutter_buffer = GlyphonBuffer::new(
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
        let bottom_panel_buffer = GlyphonBuffer::new(
            &mut font_system,
            Metrics::new(
                LayoutConstants::SMALL_FONT_SIZE,
                LayoutConstants::LINE_HEIGHT,
            ),
        );
        let activity_bar_buffer = GlyphonBuffer::new(
            &mut font_system,
            Metrics::new(24.0, LayoutConstants::ACTIVITY_BAR_WIDTH),
        );
        let overlay_buffer = GlyphonBuffer::new(
            &mut font_system,
            Metrics::new(LayoutConstants::FONT_SIZE, LayoutConstants::LINE_HEIGHT),
        );

        // Init rectangle renderer
        let rect_renderer = RectRenderer::new(&gpu.device, gpu.format());

        // Init DockTree
        let dock_tree = crate::dock::DockTree::default_layout();
        let dock_map = dock_tree.compute_layout(crate::rect_renderer::Rect {
            x: 0.0,
            y: 0.0,
            width: width as f32,
            height: height as f32,
            color: [0.0; 4],
        });

        // Compute layout from dock
        let layout = LayoutZones::from_dock_layout(dock_map, width as f32, height as f32);

        // Init UI components
        let tab_bar = TabBar::new();
        let activity_bar = ActivityBar::new();
        let gutter = Gutter::new();
        let status_bar_state = StatusBar::new();
        let cursor_renderer = CursorRenderer::new();
        let mut breadcrumb_bar = BreadcrumbBar::new();
        let scrollbar = Scrollbar::new();

        if let Some(path) = &self.file_path {
            // let filename = std::path::Path::new(path)
            //     .file_name()
            //     .map(|n| n.to_string_lossy().to_string())
            //     .unwrap_or_else(|| "untitled".to_string());
            // tab_bar.open_tab(filename, Some(path.clone()));
            breadcrumb_bar.update_from_path(path);
        }

        // Organism heartbeat
        let organism_state = organism::new_shared_state();
        let _heartbeat = organism::start_heartbeat(
            organism_state.clone(),
            std::time::Duration::from_millis(250),
        );

        let (event_tx, event_rx) = std::sync::mpsc::channel();

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
            split_editor_buffer,
            gutter_buffer,
            split_gutter_buffer,
            tab_buffer,
            status_buffer,
            breadcrumb_buffer,
            sidebar_buffer,
            bottom_panel_buffer,
            activity_bar_buffer,
            overlay_buffer,
            tab_manager,
            file_explorer,
            terminal: None,
            bottom_panel_focused: false,
            agent: Some(Agent::start()),
            debug_ui: self
                .debug_ui
                .take()
                .unwrap_or_else(|| crate::debug_ui::DebugUi::new()),
            debug_toolbar: crate::debug_toolbar::DebugToolbar::new(),
            rect_renderer,
            layout,
            tab_bar,
            activity_bar,
            gutter,
            status_bar_state,
            cursor_renderer,
            breadcrumb_bar,
            scrollbar,
            sidebar_open: false,
            ai_panel_open: false,
            last_mouse_position: None,
            frame_timer: FrameTimer::new(),
            render_batch: RenderBatch::new(),
            organism_state,
            sidebar_mode: crate::ui::SidebarMode::Explorer,
            accessibility_manager,
            debug_zones: self.debug_zones,
            dock_tree,
            event_tx,
            event_rx,
        });
    }

    /// Handle application events (async callbacks)
    fn handle_app_events(state: &mut AppState) {
        while let Ok(event) = state.event_rx.try_recv() {
            match event {
                AppEvent::GoToLocation(loc) => {
                    // Convert lsp_types::Uri to url::Url manually to access to_file_path
                    let uri_str = loc.uri.as_str();
                    if let Ok(url) = url::Url::parse(uri_str) {
                        if let Ok(path) = url.to_file_path() {
                            let path_str = path.to_string_lossy().to_string();
                            if let Err(e) = state.tab_manager.open_file(&path_str) {
                                tracing::error!("Failed to open definition file: {}", e);
                            } else if let Some(ed) = state.tab_manager.active_editor_mut() {
                                // Move cursor
                                let line = loc.range.start.line as usize;
                                let col = loc.range.start.character as usize;
                                let offset = ed.buffer.line_col_to_offset(line, col);
                                ed.buffer.set_selection(forge_core::Selection::point(
                                    forge_core::Position::new(offset),
                                ));
                                ed.set_scroll_top(line.saturating_sub(5));
                                tracing::info!("Jumped to definition: {}:{}", path_str, line);
                            }
                        } else {
                            tracing::warn!("GoToDef: Could not convert URL to file path: {}", url);
                        }
                    } else {
                        tracing::warn!("GoToDef: Invalid URI: {}", uri_str);
                    }
                }
            }
        }
    }

    /// Main render function
    fn render(
        extension_host: &mut Option<forge_extension_host::ExtensionHost>,
        state: &mut AppState,
        mode: &UiMode,
        theme: &forge_theme::Theme,
        bottom_panel: &mut crate::bottom_panel::BottomPanel,
        find_bar: &crate::find_bar::FindBar,
        replace_bar: &crate::replace_bar::ReplaceBar,
        go_to_line: &crate::go_to_line::GoToLine,
        command_palette: &crate::command_palette::CommandPalette,
        search_panel: &mut crate::search_panel::SearchPanel,
        settings_ui: &crate::settings_ui::SettingsUi,
        notifications: &mut crate::notifications::NotificationManager,
        context_menu: &crate::context_menu::ContextMenu,
        screenshot_path: Option<&String>,
    ) {
        state.frame_timer.begin_frame();
        notifications.tick();
        Self::handle_app_events(state);

        // Update accessibility tree
        let (editor_text, cursor_line, cursor_col) = if let Some(ed) = state.tab_manager.active_editor() {
            (ed.buffer.text(), ed.cursor_line(), ed.cursor_col())
        } else {
            ("".to_string(), 0, 0)
        };
        let acc_state = self::accessibility::AccessibilityState {
            editor_text,
            cursor_line,
            cursor_col,
            sidebar_visible: state.sidebar_open,
        };
        state.accessibility_manager.update(acc_state);

        // Poll Extension Host
        if let Some(host) = extension_host {
            for msg in host.poll_messages() {
                match msg {
                    forge_extension_host::protocol::HostMessage::ShowInfo { message } => {
                        notifications.show(&message, crate::notifications::Level::Info);
                    }
                    forge_extension_host::protocol::HostMessage::ShowError { message } => {
                        notifications.show(&message, crate::notifications::Level::Error);
                    }
                }
            }
        }

        // Poll AI Agent
        if let Some(agent) = &mut state.agent {
            while let Some(response) = agent.poll_response() {
                match response {
                    AgentResponse::StatusChange(status) => {
                        state.status_bar_state.ai_status = status.to_string();
                    }
                    AgentResponse::Complete(resp) => {
                        notifications.show(
                            &format!("AI: {}", resp.content),
                            crate::notifications::Level::Info,
                        );
                    }
                    AgentResponse::Error(e) => {
                        notifications.show(
                            &format!("AI Error: {}", e),
                            crate::notifications::Level::Error,
                        );
                    }
                    _ => {}
                }
            }
        }

        let (width, height) = state.gpu.size();
        if width == 0 || height == 0 {
            return;
        }

        let mode_config = mode.layout_config();

        // ─── COLLECT RECTANGLES ───
        state.render_batch.clear();

        // Background rectangles
        let bg_rects = state.layout.background_rects(theme);
        state.render_batch.extend(&bg_rects);

        // Tab bar
        if mode_config.tab_bar {
            let tab_rects = state.tab_bar.render_rects(&state.layout.tab_bar, theme);
            state.render_batch.extend(&tab_rects);
        }

        // Activity bar
        if mode_config.activity_bar {
            let ab_rects = state
                .activity_bar
                .render_rects(&state.layout.activity_bar, theme);
            state.render_batch.extend(&ab_rects);
        }

        // Gutter
        if mode_config.gutter {
            if let Some(editor) = state.tab_manager.active_editor() {
                state.gutter.scroll_top = editor.scroll_top();
                state.gutter.total_lines = editor.total_lines();
                state.gutter.cursor_line = editor.cursor_line();
            } else {
                state.gutter.scroll_top = 0;
                state.gutter.total_lines = 1;
                state.gutter.cursor_line = 0;
            }
            let gutter_rects = state.gutter.render_rects(&state.layout.gutter);
            state.render_batch.extend(&gutter_rects);
        }

        // Current line highlight
        if let Some(editor) = state.tab_manager.active_editor() {
            if let Some(hl_rect) = state.cursor_renderer.current_line_rect(
                editor.cursor_line(),
                editor.scroll_top(),
                &state.layout.editor,
            ) {
                state.render_batch.push(hl_rect);
            }
        }

        // Cursor
        if mode_config.cursor_blink {
            state.cursor_renderer.update();
        }
        if let Some(editor) = state.tab_manager.active_editor() {
            if let Some(cursor_rect) = state.cursor_renderer.render_rect(
                editor.cursor_line(),
                editor.cursor_col(),
                editor.scroll_top(),
                &state.layout.editor,
            ) {
                state.render_batch.push(cursor_rect);
            }
        }

        // Scrollbar
        let visible_lines = (state.layout.editor.height / LayoutConstants::LINE_HEIGHT) as usize;
        let (total_lines, scroll_top) = state
            .tab_manager
            .active_editor()
            .map(|e| (e.total_lines(), e.scroll_top()))
            .unwrap_or((1, 0));
        let sb_rects = state.scrollbar.render_rect(
            &state.layout.scrollbar_v,
            total_lines,
            visible_lines,
            scroll_top,
        );
        state.render_batch.extend(&sb_rects);

        // Breadcrumb
        if mode_config.breadcrumbs {
            let bc_rects = state
                .breadcrumb_bar
                .render_rects(&state.layout.breadcrumb_bar);
            state.render_batch.extend(&bc_rects);
        }

        // Bracket Match
        if let Some(editor) = state.tab_manager.active_editor() {
            let cursor_line = editor.cursor_line();
            let cursor_col = editor.cursor_col();
            let text = editor.buffer.text();
            if let Some((match_line, match_col)) =
                crate::bracket_match::BracketMatcher::find_match(&text, cursor_line, cursor_col)
            {
                if let Some(rect) = state.cursor_renderer.render_rect(
                    match_line,
                    match_col,
                    editor.scroll_top(),
                    &state.layout.editor,
                ) {
                    let mut match_rect = rect;
                    match_rect.color = [0.5, 0.5, 0.5, 0.5];
                    state.render_batch.push(match_rect);
                }
            }
        }

        // Phase 1: Find Match Highlights
        if find_bar.visible && !find_bar.matches.is_empty() {
            if let Some(editor) = state.tab_manager.active_editor() {
                let scroll_top = editor.scroll_top();
                for (i, m) in find_bar.matches.iter().enumerate() {
                    // Only highlight matches in visible viewport
                    if m.line >= scroll_top && m.line < scroll_top + visible_lines {
                        let rel_line = m.line - scroll_top;
                        let char_w = 9.2f32; // approx monospace char width
                        let match_x = state.layout.editor.x + (m.start_col as f32 * char_w);
                        let match_y = state.layout.editor.y
                            + (rel_line as f32 * LayoutConstants::LINE_HEIGHT);
                        let match_w = ((m.end_col - m.start_col) as f32) * char_w;

                        let is_current = find_bar.current_match == Some(i);
                        let color = if is_current {
                            [1.0, 0.6, 0.0, 0.4] // orange for current
                        } else {
                            [1.0, 1.0, 0.0, 0.2] // yellow for others
                        };

                        state.render_batch.push(crate::rect_renderer::Rect {
                            x: match_x,
                            y: match_y,
                            width: match_w,
                            height: LayoutConstants::LINE_HEIGHT,
                            color,
                        });
                    }
                }
            }
        }

        // Find Bar Overlay
        if find_bar.visible {
            let fb_width = 400.0;
            let fb_height = 36.0;
            let fb_x = state.layout.editor.x + state.layout.editor.width - fb_width - 20.0;
            let fb_y = state.layout.editor.y + 4.0;
            let bg = theme
                .color("editorWidget.background")
                .unwrap_or([0.18, 0.20, 0.26, 0.98]);
            state.render_batch.push(crate::rect_renderer::Rect {
                x: fb_x,
                y: fb_y,
                width: fb_width,
                height: fb_height,
                color: bg,
            });
        }

        // Replace Bar Overlay
        if replace_bar.visible {
            let fb_width = 400.0;
            let fb_height = 36.0;
            let fb_x = state.layout.editor.x + state.layout.editor.width - fb_width - 20.0;
            let fb_y = state.layout.editor.y + 44.0;
            let bg = theme
                .color("editorWidget.background")
                .unwrap_or([0.18, 0.20, 0.26, 0.98]);
            state.render_batch.push(crate::rect_renderer::Rect {
                x: fb_x,
                y: fb_y,
                width: fb_width,
                height: fb_height,
                color: bg,
            });
        }

        // Go To Line Overlay
        if go_to_line.visible {
            let g_width = 240.0;
            let g_height = 36.0;
            let g_x = state.layout.editor.x + state.layout.editor.width - g_width - 20.0;
            let g_y = state.layout.editor.y + 4.0;
            let bg = theme
                .color("editorWidget.background")
                .unwrap_or([0.18, 0.20, 0.26, 0.98]);
            state.render_batch.push(crate::rect_renderer::Rect {
                x: g_x,
                y: g_y,
                width: g_width,
                height: g_height,
                color: bg,
            });
        }

        // Command Palette Overlay
        if command_palette.visible {
            let cp_width = 500.0;
            let cp_height = 300.0;
            let (w, _h) = state.gpu.size();
            let cp_x = (w as f32 - cp_width) / 2.0;
            let cp_y = 80.0;
            let bg = theme
                .color("quickInput.background")
                .unwrap_or([0.14, 0.15, 0.20, 0.98]);
            state.render_batch.push(crate::rect_renderer::Rect {
                x: cp_x,
                y: cp_y,
                width: cp_width,
                height: cp_height,
                color: bg,
            });
        }

        // Settings UI Overlay
        if settings_ui.visible {
            let (w, h) = state.gpu.size();
            let s_width = 600.0;
            let s_height = 400.0;
            let s_x = (w as f32 - s_width) / 2.0;
            let s_y = (h as f32 - s_height) / 2.0;
            let bg = theme
                .color("editorWidget.background")
                .unwrap_or([0.14, 0.15, 0.20, 0.98]);
            state.render_batch.push(crate::rect_renderer::Rect {
                x: s_x,
                y: s_y,
                width: s_width,
                height: s_height,
                color: bg,
            });
        }

        // Debug Toolbar
        if state.debug_ui.session.active {
            let (w, _) = state.gpu.size();
            let rects = state.debug_toolbar.render_rects(w as f32);
            state.render_batch.extend(&rects);
        }

        // Context Menu
        if context_menu.visible {
            let cm_width = 200.0;
            let cm_height = (context_menu.items.len() as f32 * 24.0).max(24.0);
            let bg = theme
                .color("menu.background")
                .unwrap_or([0.18, 0.18, 0.18, 1.0]);
            state.render_batch.push(crate::rect_renderer::Rect {
                x: context_menu.x,
                y: context_menu.y,
                width: cm_width,
                height: cm_height,
                color: bg,
            });
        }

        // Notifications
        if !notifications.notifications.is_empty() {
            let (w, h) = state.gpu.size();
            let n_width = 300.0;
            let mut y_off = h as f32 - 40.0;
            for note in &notifications.notifications {
                let n_height = 60.0; // Fixed for now
                y_off -= n_height + 10.0;
                let bg = match note.level {
                    crate::notifications::Level::Error => theme
                        .color("notificationsErrorIcon.foreground")
                        .unwrap_or([0.8, 0.2, 0.2, 1.0]),
                    crate::notifications::Level::Warning => theme
                        .color("notificationsWarningIcon.foreground")
                        .unwrap_or([0.8, 0.6, 0.2, 1.0]),
                    crate::notifications::Level::Info => theme
                        .color("notificationsInfoIcon.foreground")
                        .unwrap_or([0.2, 0.4, 0.8, 1.0]),
                };
                // Background
                state.render_batch.push(crate::rect_renderer::Rect {
                    x: w as f32 - n_width - 20.0,
                    y: y_off,
                    width: n_width,
                    height: n_height,
                    color: [0.15, 0.15, 0.15, 0.95],
                });
                // Stripe
                state.render_batch.push(crate::rect_renderer::Rect {
                    x: w as f32 - n_width - 20.0,
                    y: y_off,
                    width: 4.0,
                    height: n_height,
                    color: bg,
                });
            }
        }

        // Debug Zones Overlay
        if state.debug_zones {
            forge_test_tools::zone_debug::enable_zone_overlay(state);
        }

        // Upload rectangles
        state
            .rect_renderer
            .prepare(&state.gpu.queue, &state.render_batch.rects);

        // ─── TEXT CONTENT ───

        // 1. Editor text
        let vis_lines = (state.layout.editor.height / LayoutConstants::LINE_HEIGHT) as usize + 1;
        let mut editor_text = String::new();

        if let Some(editor) = state.tab_manager.active_editor() {
            if editor.buffer.is_loading {
                editor_text = "\n\n   Loading large file...".to_string();
            } else {
                let scroll_top = editor.scroll_top();
                let total_lines = editor.total_lines();
                for i in 0..vis_lines {
                    let line_idx = scroll_top + i;
                    if line_idx >= total_lines {
                        break;
                    }
                    let line = Guard::get_line(editor.buffer.rope(), line_idx);
                    editor_text.push_str(&line);
                    if !line.ends_with('\n') {
                        editor_text.push('\n');
                    }
                }
            }
        } else {
            editor_text = "\n\n\n\
                \t\t\t🔥 FORGE EDITOR\n\n\
                \t\t\tGPU-Accelerated Code Editing\n\n\
                \t\t\tCtrl+O    Open File\n\
                \t\t\tCtrl+P    Quick Open\n\
                \t\t\tCtrl+`    Toggle Terminal\n\
                \t\t\tCtrl+,    Settings\n"
                .to_string();
        }

        state.editor_buffer.set_size(
            &mut state.font_system,
            Some(state.layout.editor.width),
            Some(state.layout.editor.height),
        );
        let text_color = theme
            .color("editor.foreground")
            .map(|c| {
                GlyphonColor::rgb(
                    (c[0] * 255.0) as u8,
                    (c[1] * 255.0) as u8,
                    (c[2] * 255.0) as u8,
                )
            })
            .unwrap_or(GlyphonColor::rgb(212, 212, 212));

        // ─── SYNTAX HIGHLIGHTING via set_rich_text ───
        // Build per-span colored text chunks from the editor's highlight_spans
        let base_attrs = Attrs::new().family(Family::Monospace).color(text_color);
        let has_highlights = state
            .tab_manager
            .active_editor()
            .map(|e| !e.highlight_spans.is_empty())
            .unwrap_or(false);

        if has_highlights {
            // Get the byte offset of the first visible line so we can map spans
            let scroll_top = state
                .tab_manager
                .active_editor()
                .map(|e| e.scroll_top())
                .unwrap_or(0);
            let (vis_byte_start, spans_clone) = {
                if let Some(editor) = state.tab_manager.active_editor() {
                    let full_text = editor.buffer.text();
                    let byte_start: usize = full_text
                        .lines()
                        .take(scroll_top)
                        .map(|l| l.len() + 1) // +1 for newline
                        .sum();
                    (byte_start, editor.highlight_spans.clone())
                } else {
                    (0, Vec::new())
                }
            };
            let vis_byte_end = vis_byte_start + editor_text.len();

            // Build rich text spans
            let mut rich_spans: Vec<(String, Attrs)> = Vec::new();
            let mut pos = 0usize; // position within editor_text

            for span in &spans_clone {
                // Skip spans before visible area
                if span.end_byte <= vis_byte_start || span.start_byte >= vis_byte_end {
                    continue;
                }
                // Clamp to visible area
                let s = span.start_byte.max(vis_byte_start) - vis_byte_start;
                let e = span.end_byte.min(vis_byte_end) - vis_byte_start;
                let s = s.min(editor_text.len());
                let e = e.min(editor_text.len());
                if s < e {
                    // Push plain text before this span
                    if pos < s {
                        rich_spans.push((editor_text[pos..s].to_string(), base_attrs));
                    }
                    // Push colored span
                    let [r, g, b] = default_color(span.token_type);
                    let color_attrs = base_attrs.color(GlyphonColor::rgb(r, g, b));
                    rich_spans.push((editor_text[s..e].to_string(), color_attrs));
                    pos = e;
                }
            }
            // Push remaining plain text
            if pos < editor_text.len() {
                rich_spans.push((editor_text[pos..].to_string(), base_attrs));
            }

            // Ghost Text (inline diff streaming)
            if let Some(ghost) = &state.tab_manager.active_editor().and_then(|e| e.ghost_text.as_ref()) {
                let ghost_attrs = base_attrs.color(GlyphonColor::rgb(136, 136, 136)).style(glyphon::Style::Italic);
                rich_spans.push((ghost.to_string(), ghost_attrs));
            }

            let rich_ref: Vec<(&str, Attrs)> =
                rich_spans.iter().map(|(s, a)| (s.as_str(), *a)).collect();
            state.editor_buffer.set_rich_text(
                &mut state.font_system,
                rich_ref,
                base_attrs,
                Shaping::Advanced,
            );
        } else {
            state.editor_buffer.set_text(
                &mut state.font_system,
                &editor_text,
                base_attrs,
                Shaping::Advanced,
            );
        }
        state
            .editor_buffer
            .shape_until_scroll(&mut state.font_system, false);

        // 2. Gutter (line numbers)
        let mut gutter_text = String::new();
        let scroll_top = state
            .tab_manager
            .active_editor()
            .map(|e| e.scroll_top())
            .unwrap_or(0);
        let total_lines = state
            .tab_manager
            .active_editor()
            .map(|e| e.total_lines())
            .unwrap_or(1);

        for i in 0..vis_lines {
            let line_idx = scroll_top + i;
            if line_idx >= total_lines {
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
        let gutter_color = theme
            .color("editorLineNumber.foreground")
            .map(|c| {
                GlyphonColor::rgb(
                    (c[0] * 255.0) as u8,
                    (c[1] * 255.0) as u8,
                    (c[2] * 255.0) as u8,
                )
            })
            .unwrap_or(GlyphonColor::rgb(133, 133, 133));

        state.gutter_buffer.set_text(
            &mut state.font_system,
            &gutter_text,
            Attrs::new().family(Family::Monospace).color(gutter_color),
            Shaping::Advanced,
        );
        state
            .gutter_buffer
            .shape_until_scroll(&mut state.font_system, false);

        // 3. Tab bar text (Deferred to dynamic buffers)
        // We will create buffers for each tab in the text area preparation phase.

        // 4. Status bar text (Deferred to dynamic buffers)
        // We will create buffers for each status item in the text area preparation phase.
        // Update status state first
        if let Some(ed) = state.tab_manager.active_editor() {
            state.status_bar_state.cursor_line = ed.cursor_line() + 1;
            state.status_bar_state.cursor_col = ed.cursor_col() + 1;
            state.status_bar_state.language = format!("{:?}", ed.language);
        }
        if state.frame_timer.avg_frame_time_ms > 0.0 {
            state.status_bar_state.frame_time_ms = state.frame_timer.avg_frame_time_ms;
        }
        state.status_bar_state.mode_indicator = format!("Mod: {}", mode.label());
        state.status_bar_state.error_count = notifications
            .notifications
            .iter()
            .filter(|n| n.level == crate::notifications::Level::Error)
            .count();
        state.status_bar_state.warning_count = notifications
            .notifications
            .iter()
            .filter(|n| n.level == crate::notifications::Level::Warning)
            .count();

        // 5. Breadcrumb text
        let breadcrumb_text = if state.breadcrumb_bar.segments.is_empty() {
            String::new() // No breadcrumb for untitled/welcome tabs
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

        // 6. Sidebar text
        if state.sidebar_open {
            let mut rich_spans: Vec<(String, Attrs)> = Vec::new();
            let header_attrs = Attrs::new()
                .family(Family::SansSerif)
                .weight(glyphon::Weight::BOLD)
                .color(GlyphonColor::rgb(187, 187, 187)); // sideBarTitle.foreground

            match state.sidebar_mode {
                crate::ui::SidebarMode::Search => {
                    rich_spans.push(("  SEARCH RESULTS\n\n".to_string(), header_attrs));
                    if search_panel.results.is_empty() {
                        let msg = if search_panel.searching {
                            "  Searching..."
                        } else {
                            "  No results."
                        };
                        rich_spans.push((
                            msg.to_string(),
                            Attrs::new()
                                .family(Family::SansSerif)
                                .color(GlyphonColor::rgb(204, 204, 204)),
                        ));
                    } else if let Some(sidebar_zone) = &state.layout.sidebar {
                        let zone = crate::ui::Zone {
                            x: sidebar_zone.x,
                            y: sidebar_zone.y + 30.0, // Offset for header
                            width: sidebar_zone.width,
                            height: sidebar_zone.height - 30.0,
                        };

                        let rects =
                            search_panel
                                .ui
                                .render_rects(&search_panel.results, &zone, theme);
                        state.render_batch.extend(&rects);

                        let line_height = 24.0;
                        let visible_count = (zone.height / line_height).ceil() as usize;
                        let scroll = search_panel.ui.scroll_offset;

                        let file_color = theme
                            .color("sideBar.foreground")
                            .unwrap_or([0.8, 0.8, 0.8, 1.0]);
                        let match_color = theme
                            .color("editor.foreground")
                            .unwrap_or([0.9, 0.9, 0.9, 1.0]);

                        let file_attrs =
                            Attrs::new()
                                .family(Family::SansSerif)
                                .color(GlyphonColor::rgb(
                                    (file_color[0] * 255.0) as u8,
                                    (file_color[1] * 255.0) as u8,
                                    (file_color[2] * 255.0) as u8,
                                ));
                        let match_attrs =
                            Attrs::new()
                                .family(Family::SansSerif)
                                .color(GlyphonColor::rgb(
                                    (match_color[0] * 255.0) as u8,
                                    (match_color[1] * 255.0) as u8,
                                    (match_color[2] * 255.0) as u8,
                                ));

                        for (_i, result) in search_panel
                            .results
                            .iter()
                            .enumerate()
                            .skip(scroll)
                            .take(visible_count)
                        {
                            let path = std::path::Path::new(&result.file);
                            let filename = path
                                .file_name()
                                .map(|s| s.to_string_lossy())
                                .unwrap_or_default();
                            rich_spans
                                .push((format!("  {}:{}", filename, result.line), file_attrs));

                            let snippet = result.text.trim();
                            let max_len = 30;
                            let display_snippet = if snippet.len() > max_len {
                                format!(" {}...", &snippet[..max_len])
                            } else {
                                format!(" {}", snippet)
                            };
                            rich_spans.push((display_snippet, match_attrs));
                            rich_spans.push(("\n".to_string(), file_attrs));
                        }
                    }
                }
                crate::ui::SidebarMode::Debug => {
                    let debug_text = state.debug_ui.render_text();
                    rich_spans.push((
                        debug_text,
                        Attrs::new()
                            .family(Family::SansSerif)
                            .color(GlyphonColor::rgb(204, 204, 204)),
                    ));
                }
                crate::ui::SidebarMode::Explorer => {
                    rich_spans.push(("  EXPLORER\n\n".to_string(), header_attrs));

                    if let Some(sidebar_zone) = &state.layout.sidebar {
                        let rects = state.file_explorer.ui.render_rects(
                            &state.file_explorer.nodes,
                            sidebar_zone,
                            2,
                        );
                        state.render_batch.extend(&rects);
                    }

                    let item_attrs = Attrs::new()
                        .family(Family::SansSerif)
                        .color(GlyphonColor::rgb(204, 204, 204));

                    let mut content = String::new();
                    for node in &state.file_explorer.nodes {
                        let indent = "  ".repeat(node.depth + 1);
                        let icon = if node.is_dir {
                            if node.expanded {
                                forge_icons::UiIcon::FolderOpen.glyph()
                            } else {
                                forge_icons::UiIcon::Folder.glyph()
                            }
                        } else {
                            forge_icons::FileIcon::from_filename(&node.label).glyph()
                        };
                        content.push_str(&format!("{}{}{}\n", indent, icon, node.label));
                    }
                    rich_spans.push((content, item_attrs));
                }
                _ => {}
            }

            if let Some(ref sb) = state.layout.sidebar {
                state.sidebar_buffer.set_size(
                    &mut state.font_system,
                    Some(sb.width - 8.0),
                    Some(sb.height),
                );
            }

            let rich_refs: Vec<(&str, Attrs)> =
                rich_spans.iter().map(|(s, a)| (s.as_str(), *a)).collect();
            state.sidebar_buffer.set_rich_text(
                &mut state.font_system,
                rich_refs,
                Attrs::new(),
                Shaping::Advanced,
            );
        } else {
            state.sidebar_buffer.set_text(
                &mut state.font_system,
                "",
                Attrs::new(),
                Shaping::Advanced,
            );
        }
        state
            .sidebar_buffer
            .shape_until_scroll(&mut state.font_system, false);

        // 7. Terminal Text
        if let Some(ref bp) = state.layout.bottom_panel {
            if bottom_panel.visible {
                let mut term_text = String::new();
                if let Some(ref mut term) = state.terminal {
                    // Render cursor and background via TerminalUi
                    let term_ui = crate::terminal_ui::TerminalUi::new();
                    let term_rects = term_ui.render_rects(term, bp);
                    state.render_batch.extend(&term_rects);

                    let _events = term.tick();
                    let grid = term.render_grid();
                    for row in &grid.cells {
                        for cell in row {
                            term_text.push(cell.ch);
                        }
                        term_text.push('\n');
                    }
                } else {
                    term_text = "Terminal not initialized (Ctrl+`)".to_string();
                }

                state.bottom_panel_buffer.set_size(
                    &mut state.font_system,
                    Some(bp.width),
                    Some(bp.height),
                );
                state.bottom_panel_buffer.set_text(
                    &mut state.font_system,
                    &term_text,
                    Attrs::new()
                        .family(Family::Monospace)
                        .color(GlyphonColor::rgb(229, 229, 229)),
                    Shaping::Advanced,
                );
                state
                    .bottom_panel_buffer
                    .shape_until_scroll(&mut state.font_system, false);
            }
        }
        // Dynamic text buffers
        let mut dynamic_buffers: Vec<GlyphonBuffer> = Vec::new();
        // Store (x, y, width, height) for each dynamic buffer
        let mut dynamic_meta: Vec<(f32, f32, f32, f32)> = Vec::new();

        // Overlay text
        if find_bar.visible {
            let current = find_bar.current_match.map(|i| i + 1).unwrap_or(0);
            let text = format!(
                "Find: {}  [{}/{}]",
                find_bar.query,
                current,
                find_bar.matches.len()
            );
            let mut buf = GlyphonBuffer::new(
                &mut state.font_system,
                Metrics::new(
                    LayoutConstants::SMALL_FONT_SIZE,
                    LayoutConstants::LINE_HEIGHT,
                ),
            );
            buf.set_text(
                &mut state.font_system,
                &text,
                Attrs::new()
                    .family(Family::SansSerif)
                    .color(GlyphonColor::rgb(220, 220, 220)),
                Shaping::Advanced,
            );
            buf.shape_until_scroll(&mut state.font_system, false);
            let fb_width = 400.0;
            let fb_height = 36.0;
            let fb_x = state.layout.editor.x + state.layout.editor.width - fb_width - 20.0;
            let fb_y = state.layout.editor.y + 4.0;
            dynamic_buffers.push(buf);
            dynamic_meta.push((fb_x + 10.0, fb_y + 10.0, fb_width - 20.0, fb_height - 8.0));
        }

        if replace_bar.visible {
            let text = format!("Replace: {}", replace_bar.replace_text);
            let mut buf = GlyphonBuffer::new(
                &mut state.font_system,
                Metrics::new(
                    LayoutConstants::SMALL_FONT_SIZE,
                    LayoutConstants::LINE_HEIGHT,
                ),
            );
            buf.set_text(
                &mut state.font_system,
                &text,
                Attrs::new()
                    .family(Family::SansSerif)
                    .color(GlyphonColor::rgb(220, 220, 220)),
                Shaping::Advanced,
            );
            buf.shape_until_scroll(&mut state.font_system, false);
            let fb_width = 400.0;
            let fb_height = 36.0;
            let fb_x = state.layout.editor.x + state.layout.editor.width - fb_width - 20.0;
            let fb_y = state.layout.editor.y + 44.0;
            dynamic_buffers.push(buf);
            dynamic_meta.push((fb_x + 10.0, fb_y + 10.0, fb_width - 20.0, fb_height - 8.0));
        }

        if go_to_line.visible {
            let text = format!("Go to line: {}", go_to_line.input);
            let mut buf = GlyphonBuffer::new(
                &mut state.font_system,
                Metrics::new(
                    LayoutConstants::SMALL_FONT_SIZE,
                    LayoutConstants::LINE_HEIGHT,
                ),
            );
            buf.set_text(
                &mut state.font_system,
                &text,
                Attrs::new()
                    .family(Family::SansSerif)
                    .color(GlyphonColor::rgb(220, 220, 220)),
                Shaping::Advanced,
            );
            buf.shape_until_scroll(&mut state.font_system, false);
            let g_width = 240.0;
            let g_height = 36.0;
            let g_x = state.layout.editor.x + state.layout.editor.width - g_width - 20.0;
            let g_y = state.layout.editor.y + 4.0;
            dynamic_buffers.push(buf);
            dynamic_meta.push((g_x + 10.0, g_y + 10.0, g_width - 20.0, g_height - 8.0));
        }

        if command_palette.visible {
            let mut cp_text = format!("> {}\n\n", command_palette.query);
            match command_palette.mode {
                crate::command_palette::PaletteMode::Commands => {
                    for (i, idx) in command_palette
                        .filtered_commands
                        .iter()
                        .take(10)
                        .enumerate()
                    {
                        if let Some(cmd) = command_palette.commands.get(*idx) {
                            let prefix = if i == 0 { ">" } else { " " };
                            cp_text.push_str(&format!("{} {}\n", prefix, cmd.label));
                        }
                    }
                }
                crate::command_palette::PaletteMode::Files => {
                    for (i, idx) in command_palette.filtered_files.iter().take(10).enumerate() {
                        if let Some(file) = command_palette.files.get(*idx) {
                            let prefix = if i == 0 { ">" } else { " " };
                            cp_text.push_str(&format!("{} {}\n", prefix, file));
                        }
                    }
                }
            }
            let mut buf = GlyphonBuffer::new(
                &mut state.font_system,
                Metrics::new(
                    LayoutConstants::SMALL_FONT_SIZE,
                    LayoutConstants::LINE_HEIGHT,
                ),
            );
            buf.set_text(
                &mut state.font_system,
                &cp_text,
                Attrs::new()
                    .family(Family::SansSerif)
                    .color(GlyphonColor::rgb(220, 220, 220)),
                Shaping::Advanced,
            );
            buf.shape_until_scroll(&mut state.font_system, false);
            let cp_width = 500.0;
            let cp_height = 300.0;
            let (w, _) = state.gpu.size();
            let cp_x = (w as f32 - cp_width) / 2.0;
            let cp_y = 80.0;
            dynamic_buffers.push(buf);
            dynamic_meta.push((cp_x + 10.0, cp_y + 10.0, cp_width - 20.0, cp_height - 20.0));
        }

        // 1. Tabs
        if !state.tab_manager.tabs.is_empty() {
            let tab_positions = state.tab_bar.text_positions(&state.layout.tab_bar, theme);
            for (text, x, y, color, _is_active, _is_mod) in tab_positions {
                let mut buf = GlyphonBuffer::new(
                    &mut state.font_system,
                    Metrics::new(
                        LayoutConstants::SMALL_FONT_SIZE,
                        LayoutConstants::LINE_HEIGHT,
                    ),
                );
                let c = GlyphonColor::rgb(
                    (color[0] * 255.0) as u8,
                    (color[1] * 255.0) as u8,
                    (color[2] * 255.0) as u8,
                );
                buf.set_text(
                    &mut state.font_system,
                    &text,
                    Attrs::new().family(Family::SansSerif).color(c),
                    Shaping::Advanced,
                );
                buf.shape_until_scroll(&mut state.font_system, false);
                dynamic_buffers.push(buf);
                dynamic_meta.push((
                    x,
                    y,
                    LayoutConstants::TAB_WIDTH,
                    LayoutConstants::TAB_BAR_HEIGHT,
                ));
            }
        } else {
            // Handle "Welcome" text in static tab_buffer
            state.tab_buffer.set_text(
                &mut state.font_system,
                "  Welcome",
                Attrs::new()
                    .family(Family::SansSerif)
                    .color(GlyphonColor::rgb(200, 200, 200)),
                Shaping::Advanced,
            );
            state
                .tab_buffer
                .shape_until_scroll(&mut state.font_system, false);
        }

        // 2. Status Bar
        let status_positions = state
            .status_bar_state
            .text_positions(&state.layout.status_bar, theme);
        for (text, x, y, color) in status_positions {
            let mut buf = GlyphonBuffer::new(
                &mut state.font_system,
                Metrics::new(
                    LayoutConstants::SMALL_FONT_SIZE,
                    LayoutConstants::LINE_HEIGHT,
                ),
            );
            let c = GlyphonColor::rgb(
                (color[0] * 255.0) as u8,
                (color[1] * 255.0) as u8,
                (color[2] * 255.0) as u8,
            );
            buf.set_text(
                &mut state.font_system,
                &text,
                Attrs::new().family(Family::SansSerif).color(c),
                Shaping::Advanced,
            );
            buf.shape_until_scroll(&mut state.font_system, false);
            dynamic_buffers.push(buf);
            // Bounds for status item are roughly infinite right
            dynamic_meta.push((x, y, 200.0, LayoutConstants::STATUS_BAR_HEIGHT));
        }

        // 3. Activity Bar
        let ab_positions = state
            .activity_bar
            .text_positions(&state.layout.activity_bar, theme);
        for (text, x, y, color) in ab_positions {
            let mut buf = GlyphonBuffer::new(
                &mut state.font_system,
                Metrics::new(24.0, LayoutConstants::ACTIVITY_BAR_WIDTH),
            );
            let c = GlyphonColor::rgb(
                (color[0] * 255.0) as u8,
                (color[1] * 255.0) as u8,
                (color[2] * 255.0) as u8,
            );
            buf.set_text(
                &mut state.font_system,
                text,
                Attrs::new().family(Family::SansSerif).color(c),
                Shaping::Advanced,
            );
            buf.shape_until_scroll(&mut state.font_system, false);
            dynamic_buffers.push(buf);
            dynamic_meta.push((
                x,
                y,
                LayoutConstants::ACTIVITY_BAR_WIDTH,
                LayoutConstants::ACTIVITY_BAR_WIDTH,
            ));
        }

        // 4. Title Bar / Menu Labels
        {
            let menu_labels = [
                "File",
                "Edit",
                "Selection",
                "View",
                "Go",
                "Run",
                "Terminal",
                "Help",
            ];
            let mut menu_x = 12.0_f32;
            let menu_y = 6.0_f32;
            let menu_fg = theme
                .color("titleBar.activeForeground")
                .unwrap_or([0.0, 0.898, 1.0, 1.0]); // cyan

            for label in &menu_labels {
                let mut buf = GlyphonBuffer::new(
                    &mut state.font_system,
                    Metrics::new(
                        LayoutConstants::SMALL_FONT_SIZE,
                        LayoutConstants::TITLE_BAR_HEIGHT,
                    ),
                );
                let c = GlyphonColor::rgb(
                    (menu_fg[0] * 255.0) as u8,
                    (menu_fg[1] * 255.0) as u8,
                    (menu_fg[2] * 255.0) as u8,
                );
                buf.set_text(
                    &mut state.font_system,
                    label,
                    Attrs::new().family(Family::SansSerif).color(c),
                    Shaping::Advanced,
                );
                buf.shape_until_scroll(&mut state.font_system, false);
                let label_width = (label.len() as f32) * 8.0 + 16.0;
                dynamic_buffers.push(buf);
                dynamic_meta.push((
                    menu_x,
                    menu_y,
                    label_width,
                    LayoutConstants::TITLE_BAR_HEIGHT,
                ));
                menu_x += label_width;
            }
        }

        // 5. Debug Toolbar Text
        if state.debug_ui.session.active {
            let (w, _) = state.gpu.size();
            let items = state.debug_toolbar.render_text(w as f32, theme);
            for (text, x, y, color) in items {
                let mut buf = GlyphonBuffer::new(
                    &mut state.font_system,
                    Metrics::new(20.0, 30.0),
                );
                let c = GlyphonColor::rgb(
                    (color[0] * 255.0) as u8,
                    (color[1] * 255.0) as u8,
                    (color[2] * 255.0) as u8,
                );
                buf.set_text(
                    &mut state.font_system,
                    &text,
                    Attrs::new().family(Family::SansSerif).color(c),
                    Shaping::Advanced,
                );
                buf.shape_until_scroll(&mut state.font_system, false);
                dynamic_buffers.push(buf);
                dynamic_meta.push((x, y, 30.0, 30.0));
            }
        }

        // 6. AI Panel text (only if panel is open)
        if let Some(ref ai_zone) = state.layout.ai_panel {
            let ai_status = if state.agent.is_some() {
                "Connected"
            } else {
                "Disconnected"
            };
            let ai_text = format!(
                "  ⬢ ANTIGRAVITY AI\n\n\
                 \x20 Status: {}\n\n\
                 \x20 Model: Claude 3.5 Sonnet\n\
                 \x20 Provider: Anthropic\n\n\
                 \x20 ─────────────────\n\n\
                 \x20 Commands:\n\
                 \x20 /chat     Open chat\n\
                 \x20 /explain  Explain code\n\
                 \x20 /fix      Fix errors\n\
                 \x20 /generate Generate code\n\
                 \x20 /refactor Refactor\n\n\
                 \x20 ─────────────────\n\n\
                 \x20 Press Ctrl+I to start\n\
                 \x20 a conversation.\n",
                ai_status
            );
            let ai_fg = theme
                .color("foreground")
                .unwrap_or([0.878, 0.871, 0.957, 1.0]);
            let mut ai_buf = GlyphonBuffer::new(
                &mut state.font_system,
                Metrics::new(
                    LayoutConstants::SMALL_FONT_SIZE,
                    LayoutConstants::LINE_HEIGHT,
                ),
            );
            ai_buf.set_size(
                &mut state.font_system,
                Some(ai_zone.width - 8.0),
                Some(ai_zone.height),
            );
            ai_buf.set_text(
                &mut state.font_system,
                &ai_text,
                Attrs::new()
                    .family(Family::SansSerif)
                    .color(GlyphonColor::rgb(
                        (ai_fg[0] * 255.0) as u8,
                        (ai_fg[1] * 255.0) as u8,
                        (ai_fg[2] * 255.0) as u8,
                    )),
                Shaping::Advanced,
            );
            ai_buf.shape_until_scroll(&mut state.font_system, false);
            dynamic_buffers.push(ai_buf);
            dynamic_meta.push((
                ai_zone.x + 4.0,
                ai_zone.y + 8.0,
                ai_zone.width - 8.0,
                ai_zone.height,
            ));
        }

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

        // Add static tab buffer if tabs are empty (for Welcome message)
        if state.tab_manager.tabs.is_empty() {
            text_areas.push(TextArea {
                buffer: &state.tab_buffer,
                left: tab.x,
                top: tab.y + 9.0,
                scale: 1.0,
                bounds: TextBounds {
                    left: tab.x as i32,
                    top: tab.y as i32,
                    right: (tab.x + tab.width) as i32,
                    bottom: (tab.y + tab.height) as i32,
                },
                default_color: GlyphonColor::rgb(200, 200, 200),
                custom_glyphs: &[],
            });
        }

        // Add dynamic buffers
        for (i, buf) in dynamic_buffers.iter().enumerate() {
            let (x, y, w, h) = dynamic_meta[i];
            text_areas.push(TextArea {
                buffer: buf,
                left: x,
                top: y,
                scale: 1.0,
                bounds: TextBounds {
                    left: x as i32,
                    top: y as i32,
                    right: (x + w) as i32,
                    bottom: (y + h) as i32,
                },
                default_color: GlyphonColor::rgb(255, 255, 255),
                custom_glyphs: &[],
            });
        }

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

        // Bottom panel (terminal) text area
        if let Some(ref bp) = state.layout.bottom_panel {
            if bottom_panel.visible {
                text_areas.push(TextArea {
                    buffer: &state.bottom_panel_buffer,
                    left: bp.x + 4.0,
                    top: bp.y + 4.0,
                    scale: 1.0,
                    bounds: TextBounds {
                        left: bp.x as i32,
                        top: bp.y as i32,
                        right: (bp.x + bp.width) as i32,
                        bottom: (bp.y + bp.height) as i32,
                    },
                    default_color: GlyphonColor::rgb(229, 229, 229),
                    custom_glyphs: &[],
                });
            }
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
        // Prepare target texture (either offscreen for screenshot or surface for display)
        let (view, offscreen_texture, surface_texture) = if screenshot_path.is_some() {
            let desc = wgpu::TextureDescriptor {
                label: Some("Screenshot Texture"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: state.gpu.config.format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            };
            let texture = state.gpu.device.create_texture(&desc);
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            (view, Some(texture), None)
        } else {
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
            (view, None, Some(surface_texture))
        };

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
                        load: wgpu::LoadOp::Clear(
                            theme
                                .color("editor.background")
                                .map(|c| wgpu::Color {
                                    r: c[0] as f64,
                                    g: c[1] as f64,
                                    b: c[2] as f64,
                                    a: c[3] as f64,
                                })
                                .unwrap_or(BG_COLOR),
                        ),
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

        // Screenshot capture logic if requested
        if let Some(path) = screenshot_path {
            let texture = offscreen_texture.as_ref().unwrap();

            // We need to copy the texture to a buffer to read it back.
            // Create buffer
            let u32_size = std::mem::size_of::<u32>() as u32;
            let output_buffer_size = (u32_size * width * height) as wgpu::BufferAddress;
            let output_buffer_desc = wgpu::BufferDescriptor {
                size: output_buffer_size,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                label: Some("Screenshot Buffer"),
                mapped_at_creation: false,
            };
            let output_buffer = state.gpu.device.create_buffer(&output_buffer_desc);

            // Copy texture to buffer
            encoder.copy_texture_to_buffer(
                wgpu::ImageCopyTexture {
                    aspect: wgpu::TextureAspect::All,
                    texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                },
                wgpu::ImageCopyBuffer {
                    buffer: &output_buffer,
                    layout: wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(u32_size * width),
                        rows_per_image: Some(height),
                    },
                },
                wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
            );

            state.gpu.queue.submit(std::iter::once(encoder.finish()));

            // Map buffer
            let buffer_slice = output_buffer.slice(..);
            let (tx, rx) = std::sync::mpsc::channel();
            buffer_slice.map_async(wgpu::MapMode::Read, move |v| {
                tx.send(v).unwrap();
            });

            state.gpu.device.poll(wgpu::Maintain::Wait);

            if let Ok(Ok(())) = rx.recv() {
                let data = buffer_slice.get_mapped_range();

                use image::{ImageBuffer, Rgba};
                let mut img_buf =
                    ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, data.to_vec()).unwrap();

                // Swap BGR to RGB if needed (assuming BGRA surface format which is common)
                // If the offscreen texture format is specifically requested as RGBA, we might not need this.
                // But surface format depends on adapter.
                // Let's assume we need swap for now.
                for pixel in img_buf.pixels_mut() {
                    let tmp = pixel[0];
                    pixel[0] = pixel[2];
                    pixel[2] = tmp;
                    pixel[3] = 255;
                }

                img_buf.save(path).unwrap();
                info!("Screenshot saved to {}", path);
            }
            output_buffer.unmap();

            // Exit after taking screenshot
            std::process::exit(0);
        } else {
            state.gpu.queue.submit(std::iter::once(encoder.finish()));
            if let Some(st) = surface_texture {
                st.present();
            }
        }

        state.frame_timer.end_frame();
    }

    fn notify_lsp(state: &AppState, rt: &Arc<Runtime>, lsp_client: &Option<Arc<LspClient>>) {
        if let Some(client) = lsp_client {
            if let Some(tab) = state.tab_manager.tabs.get(state.tab_manager.active) {
                if let Some(ref path) = tab.path {
                    if let Ok(path_abs) = std::fs::canonicalize(path) {
                        if let Ok(uri) = Url::from_file_path(path_abs) {
                            let text = tab.editor.buffer.text();
                            let client = client.clone();
                            rt.spawn(async move {
                                let _ = client.did_change(uri, 1, text).await;
                            });
                        }
                    }
                }
            }
        }
    }

    /// Handle mouse/input events on UI components
    fn handle_input(
        modifiers: ModifiersState,
        state: &mut AppState,
        event: &WindowEvent,
        bottom_panel_visible: bool,
        context_menu: &mut crate::context_menu::ContextMenu,
        search_panel: &mut crate::search_panel::SearchPanel,
        rt: &Arc<Runtime>,
        lsp_client: &Option<Arc<LspClient>>,
    ) {
        match event {
            WindowEvent::MouseInput {
                state: element_state,
                button,
                ..
            } => {
                if *element_state == ElementState::Pressed {
                    if let Some((mx, my)) = state.last_mouse_position {
                        // Search Panel Click
                        if state.sidebar_open && search_panel.visible {
                            if let Some(sb) = &state.layout.sidebar {
                                if sb.contains(mx, my) {
                                    let zone = crate::ui::Zone {
                                        x: sb.x,
                                        y: sb.y + 30.0,
                                        width: sb.width,
                                        height: sb.height - 30.0,
                                    };
                                    if let Some(idx) = search_panel.ui.handle_click(my, &zone) {
                                        if let Some(res) = search_panel.results.get(idx) {
                                            // Open file
                                            if let Err(e) = state.tab_manager.open_file(&res.file) {
                                                tracing::error!("Failed to open file: {}", e);
                                            } else {
                                                if let Some(ed) =
                                                    state.tab_manager.active_editor_mut()
                                                {
                                                    let line = res.line.saturating_sub(1);
                                                    let offset =
                                                        ed.buffer.line_col_to_offset(line, 0);
                                                    ed.buffer.set_selection(
                                                        forge_core::Selection::point(
                                                            forge_core::Position::new(offset),
                                                        ),
                                                    );
                                                    ed.set_scroll_top(line.saturating_sub(5));
                                                }
                                            }
                                        }
                                    }
                                    return;
                                }
                            }
                        }
                        if *button == winit::event::MouseButton::Right {
                            // Show context menu
                            context_menu.show(
                                mx,
                                my,
                                crate::context_menu::ContextMenu::editor_context(),
                            );
                            return;
                        }

                        if *button == winit::event::MouseButton::Left {
                            // Hide context menu on left click
                            if context_menu.visible {
                                context_menu.hide();
                            }
                            if state.layout.activity_bar.contains(mx, my) {
                                if let Some(item) = state
                                    .activity_bar
                                    .handle_click(my, &state.layout.activity_bar)
                                {
                                    // Handle AI Agent separately (panel toggle)
                                    if item == crate::activity_bar::ActivityItem::AiAgent {
                                        state.ai_panel_open = !state.ai_panel_open;
                                        state.dock_tree.set_panel_size(crate::dock::PanelId::AiPanel, if state.ai_panel_open { 350.0 } else { 0.0 });
                                    } else {
                                        // Update sidebar mode
                                        match item {
                                            crate::activity_bar::ActivityItem::Explorer => {
                                                state.sidebar_mode =
                                                    crate::ui::SidebarMode::Explorer;
                                                // Lazy-load files on first Explorer open
                                                if state.file_explorer.nodes.is_empty() {
                                                    let cwd =
                                                        std::env::current_dir().unwrap_or_default();
                                                    let _ =
                                                        state.file_explorer.scan_directory(&cwd);
                                                }
                                            }
                                            crate::activity_bar::ActivityItem::Search => {
                                                state.sidebar_mode = crate::ui::SidebarMode::Search;
                                            }
                                            crate::activity_bar::ActivityItem::Debug => {
                                                state.sidebar_mode = crate::ui::SidebarMode::Debug;
                                            }
                                            crate::activity_bar::ActivityItem::Extensions => {
                                                state.sidebar_mode =
                                                    crate::ui::SidebarMode::Extensions;
                                            }
                                            _ => {}
                                        }

                                        // Sync sidebar visibility
                                        if state.activity_bar.active_item.is_none() {
                                            state.sidebar_open = false;
                                        } else if item != crate::activity_bar::ActivityItem::AiAgent
                                            && item != crate::activity_bar::ActivityItem::Settings
                                        {
                                            state.sidebar_open = true;
                                        }

                                        state.dock_tree.set_panel_size(crate::dock::PanelId::Sidebar, if state.sidebar_open { 250.0 } else { 0.0 });

                                        // Sync search panel visibility
                                        search_panel.visible = state.sidebar_open
                                            && state.sidebar_mode == crate::ui::SidebarMode::Search;
                                    }

                                    let (w, h) = state.gpu.size();
                                    let dock_map = state.dock_tree.compute_layout(crate::rect_renderer::Rect {
                                        x: 0.0,
                                        y: 0.0,
                                        width: w as f32,
                                        height: h as f32,
                                        color: [0.0; 4],
                                    });
                                    state.layout = LayoutZones::from_dock_layout(dock_map, w as f32, h as f32);
                                }
                            } else if state.layout.tab_bar.contains(mx, my) {
                                state.tab_bar.handle_click(mx, &state.layout.tab_bar);
                            } else if state.layout.gutter.contains(mx, my) {
                                if let Some(line) =
                                    state.gutter.handle_click(my, &state.layout.gutter)
                                {
                                    if let Some(tab) =
                                        state.tab_manager.tabs.get(state.tab_manager.active)
                                    {
                                        if let Some(ref path) = tab.path {
                                            state.debug_ui.toggle_breakpoint(
                                                path.to_string_lossy().to_string(),
                                                line,
                                                rt,
                                            );
                                        }
                                    }
                                }
                            } else if state.layout.scrollbar_v.contains(mx, my) {
                                let scroll_top = state
                                    .tab_manager
                                    .active_editor()
                                    .map(|e| e.scroll_top())
                                    .unwrap_or(0);
                                state.scrollbar.start_drag(my, scroll_top);
                            } else if state.layout.editor.contains(mx, my) {
                                // ─── Editor click → set cursor position ───
                                let editor_zone = &state.layout.editor;
                                let rel_x = mx - editor_zone.x;
                                let rel_y = my - editor_zone.y;

                                let scroll_top = state
                                    .tab_manager
                                    .active_editor()
                                    .map(|e| e.scroll_top())
                                    .unwrap_or(0);

                                let clicked_line = scroll_top
                                    + (rel_y / crate::ui::LayoutConstants::LINE_HEIGHT) as usize;
                                let clicked_col =
                                    (rel_x / crate::ui::LayoutConstants::CHAR_WIDTH) as usize;

                                if let Some(ed) = state.tab_manager.active_editor_mut() {
                                    let max_line = ed.total_lines().saturating_sub(1);
                                    let target_line = clicked_line.min(max_line);

                                    let text = ed.buffer.text();
                                    let line_text = text.lines().nth(target_line).unwrap_or("");
                                    let target_col = clicked_col.min(line_text.len());

                                    let offset =
                                        ed.buffer.line_col_to_offset(target_line, target_col);

                                    if modifiers.alt_key() {
                                        ed.add_cursor_at_point(target_line, target_col);
                                    } else {
                                        ed.buffer.set_selection(forge_core::Selection::point(
                                            forge_core::Position::new(offset),
                                        ));
                                    }
                                }
                                state.window.request_redraw();
                            } else if state.sidebar_open
                                && state.sidebar_mode == crate::ui::SidebarMode::Explorer
                            {
                                // ─── File Explorer click → open file ───
                                if let Some(ref sb) = state.layout.sidebar {
                                    if sb.contains(mx, my) {
                                        let rel_y = my - sb.y;
                                        if rel_y >= 0.0 {
                                            // FileTreeUi renders with line_h = 22.0
                                            // "EXPLORER\n\n" header: 2 lines × 22px = 44px
                                            let header_offset = 44.0_f32;
                                            let item_height = 22.0_f32;
                                            let adjusted_y = rel_y - header_offset;
                                            if adjusted_y < 0.0 {
                                                // Clicked on header, ignore
                                                state.window.request_redraw();
                                                return;
                                            }
                                            let clicked_index = (adjusted_y / item_height) as usize;

                                            if clicked_index < state.file_explorer.paths.len() {
                                                state.file_explorer.ui.selected_index =
                                                    Some(clicked_index);

                                                let is_dir = state
                                                    .file_explorer
                                                    .nodes
                                                    .get(clicked_index)
                                                    .map(|n| n.is_dir)
                                                    .unwrap_or(false);

                                                if is_dir {
                                                    state
                                                        .file_explorer
                                                        .toggle_expand(clicked_index);
                                                } else {
                                                    let path = state.file_explorer.paths
                                                        [clicked_index]
                                                        .to_string_lossy()
                                                        .to_string();
                                                    if let Err(e) =
                                                        state.tab_manager.open_file(&path)
                                                    {
                                                        tracing::error!(
                                                            "Failed to open file: {}",
                                                            e
                                                        );
                                                    } else {
                                                        // Update breadcrumb
                                                        state
                                                            .breadcrumb_bar
                                                            .update_from_path(&path);
                                                        // Notify LSP
                                                        Self::notify_lsp(state, rt, lsp_client);
                                                    }
                                                }
                                            }
                                        }
                                        state.window.request_redraw();
                                    }
                                }
                            }
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

                if state.sidebar_open
                    && state.sidebar_mode == crate::ui::SidebarMode::Explorer
                {
                    if let Some(sb) = &state.layout.sidebar {
                        if sb.contains(mx, my) {
                            let rel_y = my - sb.y;
                            let header_offset = 44.0;
                            let item_height = 22.0;
                            let adjusted_y = rel_y - header_offset;
                            if adjusted_y >= 0.0 {
                                let idx = (adjusted_y / item_height) as usize;
                                if idx < state.file_explorer.nodes.len() {
                                    state.file_explorer.ui.hovered_index = Some(idx);
                                } else {
                                    state.file_explorer.ui.hovered_index = None;
                                }
                            } else {
                                state.file_explorer.ui.hovered_index = None;
                            }
                        } else {
                            state.file_explorer.ui.hovered_index = None;
                        }
                    } else {
                        state.file_explorer.ui.hovered_index = None;
                    }
                } else {
                    state.file_explorer.ui.hovered_index = None;
                }

                if state.scrollbar.dragging {
                    let visible =
                        (state.layout.editor.height / LayoutConstants::LINE_HEIGHT) as usize;
                    let total_lines = state
                        .tab_manager
                        .active_editor()
                        .map(|e| e.total_lines())
                        .unwrap_or(1);
                    let new_scroll = state.scrollbar.update_drag(
                        my,
                        &state.layout.scrollbar_v,
                        total_lines,
                        visible,
                    );
                    if let Some(editor) = state.tab_manager.active_editor_mut() {
                        editor.set_scroll_top(new_scroll);
                    }
                }
            }
            _ => {}
        }
    }
}

// ─── winit ApplicationHandler ───

impl ApplicationHandler<UserEvent> for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_none() {
            self.init_state(event_loop);
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: UserEvent) {
        // Handle accessibility user events if needed (adapter handles them via callbacks/proxy mostly)
        // AccessKit adapter events are usually handled by the adapter itself when it gets called?
        // Actually, AccessKit uses the proxy to wake up the event loop so that we can process requests.
        // But accesskit_winit might not need explicit handling here if we don't have custom logic?
        // Wait, accesskit_winit documentation says:
        // "Your application must implement ApplicationHandler<UserEvent> ... in user_event, call adapter.process_accesskit_event"
        // But accesskit_winit 0.16 Adapter doesn't seem to have process_accesskit_event?
        // Let's check the source or docs if possible.
        // Assuming we just need to wake up.
        // Actually, checking standard implementation:
        // If we passed the proxy to adapter, it will send events. We need to handle them?
        // Let's look at accessibility.rs again.
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

        state.accessibility_manager.on_event(&state.window, &event);

        match event {
            WindowEvent::CloseRequested => {
                info!("Goodbye from Forge 🔥");
                event_loop.exit();
            }

            WindowEvent::Resized(size) => {
                if size.width > 0 && size.height > 0 {
                    state.gpu.resize(size.width, size.height);

                    let dock_map = state.dock_tree.compute_layout(crate::rect_renderer::Rect {
                        x: 0.0,
                        y: 0.0,
                        width: size.width as f32,
                        height: size.height as f32,
                        color: [0.0; 4],
                    });
                    state.layout = LayoutZones::from_dock_layout(
                        dock_map,
                        size.width as f32,
                        size.height as f32,
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

                tracing::debug!(
                    "KeyboardInput: key={:?} ctrl={} shift={}",
                    key_event.logical_key,
                    self.modifiers.control_key(),
                    self.modifiers.shift_key(),
                );

                let ctrl = self.modifiers.control_key();
                let shift = self.modifiers.shift_key();

                match key_event.logical_key {
                    Key::Named(NamedKey::Tab) if ctrl => {
                        state.tab_manager.next_tab();
                        // Phase 4: Track tab switch in ghost tabs
                        if let Some(tab) = state.tab_manager.tabs.get(state.tab_manager.active) {
                            if let Some(ref path) = tab.path {
                                self.ghost_tabs.on_file_open(&path.to_string_lossy());
                            }
                        }
                        // Phase 6: Update breadcrumb on tab switch
                        if let Some(tab) = state.tab_manager.tabs.get(state.tab_manager.active) {
                            if let Some(ref path) = tab.path {
                                state
                                    .breadcrumb_bar
                                    .update_from_path(&path.to_string_lossy());
                            }
                        }
                        state.window.request_redraw();
                    }
                    Key::Character(ref c) if ctrl => match c.as_str() {
                        "p" if shift => {
                            self.find_bar.close();
                            self.replace_bar.close();
                            self.go_to_line.cancel();
                            self.command_palette
                                .open(crate::command_palette::PaletteMode::Commands);
                            state.window.request_redraw();
                        }
                        "p" => {
                            self.find_bar.close();
                            self.replace_bar.close();
                            self.go_to_line.cancel();
                            // Collect files from file_explorer
                            // FileExplorer.paths contains all paths
                            let files: Vec<String> = state
                                .file_explorer
                                .paths
                                .iter()
                                .map(|p| p.to_string_lossy().to_string())
                                .collect();
                            self.command_palette.set_files(files);
                            self.command_palette
                                .open(crate::command_palette::PaletteMode::Files);
                            state.window.request_redraw();
                        }
                        "i" if shift => {
                            // Phase 3: Toggle AI Panel
                            state.ai_panel_open = !state.ai_panel_open;
                            state.dock_tree.set_panel_size(crate::dock::PanelId::AiPanel, if state.ai_panel_open { 350.0 } else { 0.0 });
                            let (w, h) = state.gpu.size();
                            let dock_map = state.dock_tree.compute_layout(crate::rect_renderer::Rect {
                                x: 0.0,
                                y: 0.0,
                                width: w as f32,
                                height: h as f32,
                                color: [0.0; 4],
                            });
                            state.layout = LayoutZones::from_dock_layout(dock_map, w as f32, h as f32);
                            tracing::info!(
                                "AI Panel: {}",
                                if state.ai_panel_open {
                                    "opened"
                                } else {
                                    "closed"
                                }
                            );
                            state.window.request_redraw();
                        }
                        "f" if shift => {
                            self.command_palette.close();
                            self.go_to_line.cancel();
                            self.replace_bar.close();
                            self.find_bar.close();

                            // Open Search Sidebar
                            state.sidebar_mode = crate::ui::SidebarMode::Search;
                            state.sidebar_open = true;
                            self.search_panel.visible = true;

                            let cwd = std::env::current_dir().unwrap_or_default();
                            // If selection exists, pre-fill query?
                            // For now, if query is empty, use "TODO" or just focus
                            if self.search_panel.query.is_empty() {
                                self.search_panel.query = "".to_string();
                            }
                            // Don't auto-search on open unless we have context,
                            // but existing logic did auto-search.

                            state.window.request_redraw();
                        }
                        "f" => {
                            self.command_palette.close();
                            self.go_to_line.cancel();
                            self.replace_bar.close();
                            self.find_bar.open();
                            state.window.request_redraw();
                        }
                        "h" => {
                            self.command_palette.close();
                            self.go_to_line.cancel();
                            self.find_bar.open();
                            if self.replace_bar.visible {
                                self.replace_bar.close();
                            } else {
                                self.replace_bar.open();
                            }
                            state.window.request_redraw();
                        }
                        "`" => {
                            self.bottom_panel.toggle();
                            state.bottom_panel_focused = self.bottom_panel.visible;
                            if self.bottom_panel.visible && state.terminal.is_none() {
                                match forge_terminal::Terminal::new() {
                                    Ok(term) => state.terminal = Some(term),
                                    Err(e) => tracing::warn!("Terminal failed: {}", e),
                                }
                            }
                            // Recompute layout
                            state.dock_tree.set_panel_size(crate::dock::PanelId::BottomPanel, if self.bottom_panel.visible { 200.0 } else { 0.0 });
                            let (w, h) = state.gpu.size();
                            let dock_map = state.dock_tree.compute_layout(crate::rect_renderer::Rect {
                                x: 0.0,
                                y: 0.0,
                                width: w as f32,
                                height: h as f32,
                                color: [0.0; 4],
                            });
                            state.layout = LayoutZones::from_dock_layout(dock_map, w as f32, h as f32);
                            state.window.request_redraw();
                        }
                        "," => {
                            self.settings_ui.toggle();
                            state.window.request_redraw();
                        }
                        "g" if !shift => {
                            self.command_palette.close();
                            self.find_bar.close();
                            self.replace_bar.close();
                            if self.go_to_line.visible {
                                self.go_to_line.cancel();
                            } else {
                                self.go_to_line.open();
                            }
                            state.window.request_redraw();
                        }
                        "\\" => {
                            state.tab_manager.split_current();
                            state.window.request_redraw();
                        }
                        "1" if ctrl => {
                            state.tab_manager.focused_pane = crate::tab_manager::Pane::Primary;
                            state.window.request_redraw();
                        }
                        "2" if ctrl => {
                            if state.tab_manager.active_secondary.is_some() {
                                state.tab_manager.focused_pane =
                                    crate::tab_manager::Pane::Secondary;
                                state.window.request_redraw();
                            }
                        }
                        "k" => {
                            // Zen mode temporary logic (simplified)
                            // Ideally zen mode modifies dock tree directly, but current zen implementation expects LayoutZones
                            // For now we'll just toggle panels off
                            state.sidebar_open = false;
                            state.ai_panel_open = false;
                            self.bottom_panel.visible = false;

                            state.dock_tree.set_panel_size(crate::dock::PanelId::Sidebar, 0.0);
                            state.dock_tree.set_panel_size(crate::dock::PanelId::AiPanel, 0.0);
                            state.dock_tree.set_panel_size(crate::dock::PanelId::BottomPanel, 0.0);

                            let (w, h) = state.gpu.size();
                            let dock_map = state.dock_tree.compute_layout(crate::rect_renderer::Rect {
                                x: 0.0,
                                y: 0.0,
                                width: w as f32,
                                height: h as f32,
                                color: [0.0; 4],
                            });
                            state.layout = LayoutZones::from_dock_layout(dock_map, w as f32, h as f32);
                            state.window.request_redraw();
                        }
                        "s" => {
                            // Atomic save via forge-core FileIO
                            if let Some(tab) = state.tab_manager.tabs.get(state.tab_manager.active)
                            {
                                if let Some(ref path) = tab.path {
                                    if let Some(ed) = state.tab_manager.active_editor() {
                                        let text = ed.buffer.text();
                                        if let Err(e) =
                                            forge_core::file_io::FileIO::save_atomic(path, &text)
                                        {
                                            tracing::error!("Save failed: {}", e);
                                        } else {
                                            tracing::info!("Saved: {}", path.display());
                                        }
                                    }
                                }
                            }
                            // Mark tab as not modified after save
                            if let Some(tab) =
                                state.tab_manager.tabs.get_mut(state.tab_manager.active)
                            {
                                tab.is_modified = false;
                            }
                        }
                        "w" => {
                            state.tab_manager.close_current();
                            state.window.request_redraw();
                        }
                        "z" if shift => {
                            if let Some(ed) = state.tab_manager.active_editor_mut() {
                                ed.buffer.redo();
                                ed.rehighlight();
                            }
                            Self::notify_lsp(state, &self.rt, &self.lsp_client);
                        }
                        "z" => {
                            if let Some(ed) = state.tab_manager.active_editor_mut() {
                                ed.buffer.undo();
                                ed.rehighlight(); // Ensure syntax highlighting is updated
                            }
                            Self::notify_lsp(state, &self.rt, &self.lsp_client);
                        }
                        "a" => {
                            if let Some(ed) = state.tab_manager.active_editor_mut() {
                                let len = ed.buffer.len_bytes();
                                ed.buffer.set_selection(forge_core::Selection::single(
                                    forge_core::Range::new(
                                        forge_core::Position::new(0),
                                        forge_core::Position::new(len),
                                    ),
                                ));
                            }
                            state.window.request_redraw();
                        }
                        "b" => {
                            state.sidebar_open = !state.sidebar_open;
                            let (w, h) = state.gpu.size();
                            state.layout = LayoutZones::compute(
                                w as f32,
                                h as f32,
                                state.sidebar_open,
                                state.ai_panel_open,
                                self.bottom_panel.visible,
                            );
                            state.window.request_redraw();
                        }
                        "y" => {
                            if let Some(ed) = state.tab_manager.active_editor_mut() {
                                ed.buffer.redo();
                                ed.rehighlight(); // Ensure syntax highlighting is updated
                            }
                            Self::notify_lsp(state, &self.rt, &self.lsp_client);
                        }
                        "c" => {
                            // Clipboard copy
                            if let Some(ed) = state.tab_manager.active_editor() {
                                let text = ed.buffer.text();
                                let (line, _) = ed.cursor_line_col();
                                if let Some(line_text) = text.lines().nth(line) {
                                    if let Ok(mut clipboard) = arboard::Clipboard::new() {
                                        let _ = clipboard.set_text(line_text.to_string());
                                    }
                                }
                            }
                        }
                        "d" if !shift => {
                            // Ctrl+D = Add Selection To Next Find Match
                            if let Some(ed) = state.tab_manager.active_editor_mut() {
                                ed.select_next_occurrence();
                            }
                        }
                        "v" => {
                            // Clipboard paste
                            if let Ok(mut clipboard) = arboard::Clipboard::new() {
                                if let Ok(text) = clipboard.get_text() {
                                    if let Some(ed) = state.tab_manager.active_editor_mut() {
                                        for ch in text.chars() {
                                            ed.insert_char(ch);
                                        }
                                        ed.rehighlight();
                                    }
                                    state.tab_manager.mark_active_modified();
                                    Self::notify_lsp(state, &self.rt, &self.lsp_client);
                                }
                            }
                        }
                        "x" => {
                            // Clipboard cut (copy current line + delete it)
                            if let Some(ed) = state.tab_manager.active_editor_mut() {
                                let text = ed.buffer.text();
                                let (line, _) = ed.cursor_line_col();
                                if let Some(line_text) = text.lines().nth(line) {
                                    if let Ok(mut clipboard) = arboard::Clipboard::new() {
                                        let _ = clipboard.set_text(line_text.to_string());
                                    }
                                    // Delete the line
                                    ed.delete_line(line);
                                    ed.rehighlight();
                                }
                            }
                            state.tab_manager.mark_active_modified();
                            Self::notify_lsp(state, &self.rt, &self.lsp_client);
                        }
                        "o" => {
                            // Open file dialog (simple native dialog)
                            #[cfg(target_os = "windows")]
                            {
                                // Use rfd or native-dialog if available, else log
                                tracing::info!("Ctrl+O: Open File (use command palette for now)");
                            }
                        }
                        "m" => {
                            self.current_mode = self.current_mode.next();
                        }
                        "g" if shift => {
                            // Mock AI Ghost Text Trigger (/ghost)
                            if let Some(ed) = state.tab_manager.active_editor_mut() {
                                if ed.ghost_text.is_some() {
                                    ed.ghost_text = None;
                                } else {
                                    ed.ghost_text = Some(" // AI Suggestion: Optimize this loop".to_string());
                                }
                            }
                            state.window.request_redraw();
                        }
                        "d" if shift => {
                            // Ctrl+Shift+D = Go to Definition
                            if let Some(tab) = state.tab_manager.tabs.get(state.tab_manager.active)
                            {
                                if let Some(ref path) = tab.path {
                                    let (line, col) = tab.editor.cursor_line_col();
                                    crate::go_to_def::GoToDef::execute(
                                        &self.rt,
                                        &self.lsp_client,
                                        &path.to_string_lossy(),
                                        line as u32,
                                        col as u32,
                                        &mut self.notifications,
                                        state.event_tx.clone(),
                                    );
                                }
                            }
                        }
                        "e" if shift => {
                            // Ctrl+Shift+E = Explain
                            if let Some(agent) = &state.agent {
                                if let Some(ed) = state.tab_manager.active_editor() {
                                    let selection =
                                        if !ed.buffer.selection().is_empty() {
                                            let range = ed.buffer.selection().primary();
                                            if range.is_empty() {
                                                None
                                            } else {
                                                Some(ed.buffer.slice(
                                                    range.start().offset,
                                                    range.end().offset,
                                                ))
                                            }
                                        } else {
                                            None
                                        };

                                    let context = EditorContext {
                                        file_path: state
                                            .tab_manager
                                            .tabs
                                            .get(state.tab_manager.active)
                                            .and_then(|t| t.path.as_ref())
                                            .map(|p| p.to_string_lossy().to_string()),
                                        file_content: Some(ed.buffer.text()),
                                        cursor_line: ed.cursor_line(),
                                        cursor_col: ed.cursor_col(),
                                        selection,
                                        language: format!("{:?}", ed.language),
                                        confidence_score: None,
                                    };
                                    agent.send_slash_command("/explain".to_string(), context);
                                    self.notifications.show(
                                        "AI: Explaining...",
                                        crate::notifications::Level::Info,
                                    );
                                }
                            }
                        }
                        _ => {}
                    },

                    Key::Named(NamedKey::ArrowLeft) => {
                        if let Some(ed) = state.tab_manager.active_editor_mut() {
                            ed.move_left();
                        }
                    }
                    Key::Named(NamedKey::ArrowRight) => {
                        if let Some(ed) = state.tab_manager.active_editor_mut() {
                            ed.move_right();
                        }
                    }
                    Key::Named(NamedKey::ArrowUp) => {
                        if let Some(ed) = state.tab_manager.active_editor_mut() {
                            ed.move_up();
                        }
                    }
                    Key::Named(NamedKey::ArrowDown) => {
                        if let Some(ed) = state.tab_manager.active_editor_mut() {
                            ed.move_down();
                        }
                    }
                    Key::Named(NamedKey::Home) => {
                        if let Some(ed) = state.tab_manager.active_editor_mut() {
                            ed.move_home();
                        }
                    }
                    Key::Named(NamedKey::End) => {
                        if let Some(ed) = state.tab_manager.active_editor_mut() {
                            ed.move_end();
                        }
                    }
                    Key::Named(NamedKey::PageUp) => {
                        if let Some(ed) = state.tab_manager.active_editor_mut() {
                            ed.scroll(-30.0);
                        }
                        state.window.request_redraw();
                    }
                    Key::Named(NamedKey::PageDown) => {
                        if let Some(ed) = state.tab_manager.active_editor_mut() {
                            ed.scroll(30.0);
                        }
                        state.window.request_redraw();
                    }

                    Key::Named(NamedKey::Backspace) => {
                        if self.command_palette.visible {
                            self.command_palette.backspace();
                        } else if self.go_to_line.visible {
                            self.go_to_line.input.pop();
                        } else if self.replace_bar.visible {
                            self.replace_bar.replace_text.pop();
                        } else if self.find_bar.visible {
                            self.find_bar.query.pop();
                            if let Some(ed) = state.tab_manager.active_editor() {
                                let text = ed.buffer.text();
                                let query = self.find_bar.query.clone();
                                self.find_bar.search(&text, &query);
                            }
                        } else {
                            if let Some(ed) = state.tab_manager.active_editor_mut() {
                                ed.backspace();
                                ed.rehighlight();
                            }
                            state.tab_manager.mark_active_modified();
                            Self::notify_lsp(state, &self.rt, &self.lsp_client);
                        }
                    }
                    Key::Named(NamedKey::Delete) => {
                        if let Some(ed) = state.tab_manager.active_editor_mut() {
                            ed.delete();
                            ed.rehighlight();
                        }
                        state.tab_manager.mark_active_modified();
                        Self::notify_lsp(state, &self.rt, &self.lsp_client);
                    }
                    Key::Named(NamedKey::Enter) => {
                        if self.command_palette.visible {
                            match self.command_palette.mode {
                                crate::command_palette::PaletteMode::Commands => {
                                    if let Some(cmd) = self.command_palette.select_command(0) {
                                        let cmd_id = cmd.id.clone();
                                        let cmd_label = cmd.label.clone();
                                        self.command_palette.close();
                                        match cmd_id.as_str() {
                                            "edit.find" => {
                                                self.find_bar.open();
                                                self.go_to_line.cancel();
                                                self.replace_bar.close();
                                            }
                                            "edit.replace" => {
                                                self.find_bar.open();
                                                self.go_to_line.cancel();
                                                self.replace_bar.open();
                                            }
                                            "edit.find_in_files" => {
                                                state.sidebar_mode = crate::ui::SidebarMode::Search;
                                                state.sidebar_open = true;
                                                self.search_panel.visible = true;

                                                let cwd =
                                                    std::env::current_dir().unwrap_or_default();
                                                self.search_panel.query = "TODO".to_string();
                                                self.search_panel.search(&cwd);

                                                state.window.request_redraw();
                                            }
                                            "view.terminal" => {
                                                self.bottom_panel.toggle();
                                                state.bottom_panel_focused =
                                                    self.bottom_panel.visible;
                                                if self.bottom_panel.visible
                                                    && state.terminal.is_none()
                                                {
                                                    match forge_terminal::Terminal::new() {
                                                        Ok(term) => state.terminal = Some(term),
                                                        Err(e) => {
                                                            tracing::warn!("Terminal failed: {}", e)
                                                        }
                                                    }
                                                }
                                                let (w, h) = state.gpu.size();
                                                state.layout = LayoutZones::compute(
                                                    w as f32,
                                                    h as f32,
                                                    state.sidebar_open,
                                                    state.ai_panel_open,
                                                    self.bottom_panel.visible,
                                                );
                                            }
                                            "view.sidebar" => {
                                                state.sidebar_open = !state.sidebar_open;
                                                let (w, h) = state.gpu.size();
                                                state.layout = LayoutZones::compute(
                                                    w as f32,
                                                    h as f32,
                                                    state.sidebar_open,
                                                    state.ai_panel_open,
                                                    self.bottom_panel.visible,
                                                );
                                            }
                                            "file.save" => {
                                                if let Some(tab) = state
                                                    .tab_manager
                                                    .tabs
                                                    .get(state.tab_manager.active)
                                                {
                                                    if let Some(ref path) = tab.path {
                                                        if let Some(ed) =
                                                            state.tab_manager.active_editor()
                                                        {
                                                            let text = ed.buffer.text();
                                                            if let Err(e) =
                                                        forge_core::file_io::FileIO::save_atomic(
                                                            path, &text,
                                                        )
                                                    {
                                                        tracing::error!("Save failed: {}", e);
                                                    }
                                                        }
                                                    }
                                                }
                                                if let Some(tab) = state
                                                    .tab_manager
                                                    .tabs
                                                    .get_mut(state.tab_manager.active)
                                                {
                                                    tab.is_modified = false;
                                                }
                                            }
                                            "file.close" => {
                                                state.tab_manager.close_current();
                                            }
                                            _ => {
                                                self.notifications.show(
                                                    &format!(
                                                        "Command '{}' is not wired yet.",
                                                        cmd_label
                                                    ),
                                                    crate::notifications::Level::Info,
                                                );
                                            }
                                        }
                                    } else {
                                        self.command_palette.close();
                                    }
                                }
                                crate::command_palette::PaletteMode::Files => {
                                    if let Some(file) = self.command_palette.select_file(0) {
                                        let path = file.clone();
                                        self.command_palette.close();
                                        if let Err(e) = state.tab_manager.open_file(&path) {
                                            tracing::error!("Failed to open file: {}", e);
                                        }
                                        state.window.request_redraw();
                                        // Notify LSP
                                        Self::notify_lsp(state, &self.rt, &self.lsp_client);
                                    } else {
                                        self.command_palette.close();
                                    }
                                }
                            }
                        } else if self.go_to_line.visible {
                            if let Some((line, col_opt)) = self.go_to_line.confirm() {
                                if let Some(ed) = state.tab_manager.active_editor_mut() {
                                    let max_line = ed.total_lines().saturating_sub(1);
                                    let target_line = line.min(max_line);
                                    let target_col = col_opt.unwrap_or(0);
                                    let line_start = ed.buffer.line_col_to_offset(target_line, 0);
                                    let text = ed.buffer.text();
                                    let line_len =
                                        text[line_start..].lines().next().unwrap_or("").len();
                                    let target_col = target_col.min(line_len);
                                    let offset =
                                        ed.buffer.line_col_to_offset(target_line, target_col);
                                    ed.buffer.set_selection(forge_core::Selection::point(
                                        forge_core::Position::new(offset),
                                    ));
                                    ed.set_scroll_top(target_line.saturating_sub(5));
                                }
                            }
                            self.go_to_line.cancel();
                        } else if self.replace_bar.visible {
                            let current_match = self
                                .find_bar
                                .current_match
                                .and_then(|idx| self.find_bar.matches.get(idx).cloned());
                            if let Some(m) = current_match {
                                if let Some(ed) = state.tab_manager.active_editor_mut() {
                                    let start = ed.buffer.line_col_to_offset(m.line, m.start_col);
                                    let end = ed.buffer.line_col_to_offset(m.line, m.end_col);
                                    let replacement = self.replace_bar.replace_text.clone();
                                    let change = forge_core::Change::replace(
                                        forge_core::Position::new(start),
                                        forge_core::Position::new(end),
                                        replacement.clone(),
                                    );
                                    let tx = forge_core::Transaction::new(
                                        forge_core::ChangeSet::with_change(change),
                                        Some(forge_core::Selection::point(
                                            forge_core::Position::new(start + replacement.len()),
                                        )),
                                    );
                                    ed.buffer.apply(tx);
                                    ed.rehighlight();
                                    let text = ed.buffer.text();
                                    let query = self.find_bar.query.clone();
                                    self.find_bar.search(&text, &query);
                                }
                                if let Some(tab) =
                                    state.tab_manager.tabs.get_mut(state.tab_manager.active)
                                {
                                    tab.is_modified = true;
                                }
                                Self::notify_lsp(state, &self.rt, &self.lsp_client);
                            } else {
                                self.notifications.show(
                                    "No active match to replace.",
                                    crate::notifications::Level::Info,
                                );
                            }
                        } else if self.find_bar.visible {
                            // Navigate to next match when Enter is pressed in find bar
                            if let Some(m) = self.find_bar.next_match() {
                                let target_line = m.line;
                                if let Some(ed) = state.tab_manager.active_editor_mut() {
                                    let offset = ed.buffer.line_col_to_offset(target_line, 0);
                                    ed.buffer.set_selection(forge_core::Selection::point(
                                        forge_core::Position::new(offset),
                                    ));
                                    ed.set_scroll_top(target_line.saturating_sub(5));
                                }
                            }
                        } else {
                            if let Some(ed) = state.tab_manager.active_editor_mut() {
                                ed.insert_newline();
                                ed.rehighlight();
                            }
                            state.tab_manager.mark_active_modified();
                            Self::notify_lsp(state, &self.rt, &self.lsp_client);
                        }
                    }
                    Key::Named(NamedKey::Escape) => {
                        if self.find_bar.visible {
                            self.find_bar.close();
                        }
                        if self.replace_bar.visible {
                            self.replace_bar.close();
                        }
                        if self.go_to_line.visible {
                            self.go_to_line.cancel();
                        }
                        if self.command_palette.visible {
                            self.command_palette.close();
                        }
                        if self.settings_ui.visible {
                            self.settings_ui.toggle();
                        }
                    }
                    Key::Named(NamedKey::Tab) => {
                        if let Some(ed) = state.tab_manager.active_editor_mut() {
                            for _ in 0..4 {
                                ed.insert_char(' ');
                            }
                            ed.rehighlight();
                        }
                        state.tab_manager.mark_active_modified();
                        Self::notify_lsp(state, &self.rt, &self.lsp_client);
                    }

                    Key::Character(ref c) if !ctrl => {
                        if self.command_palette.visible {
                            for ch in c.chars() {
                                self.command_palette.type_char(ch);
                            }
                        } else if self.go_to_line.visible {
                            for ch in c.chars() {
                                if ch.is_ascii_digit() || ch == ':' {
                                    self.go_to_line.type_char(ch);
                                }
                            }
                        } else if self.replace_bar.visible {
                            self.replace_bar.replace_text.push_str(c);
                        } else if self.find_bar.visible {
                            self.find_bar.query.push_str(c);
                            // Live search as user types
                            if let Some(ed) = state.tab_manager.active_editor() {
                                let text = ed.buffer.text();
                                let query = self.find_bar.query.clone();
                                self.find_bar.search(&text, &query);
                            }
                        } else {
                            if let Some(ed) = state.tab_manager.active_editor_mut() {
                                for ch in c.chars() {
                                    ed.insert_char(ch);
                                }
                                ed.rehighlight();
                            }
                            // Mark tab as modified
                            state.tab_manager.mark_active_modified();
                            Self::notify_lsp(state, &self.rt, &self.lsp_client);
                        }
                    }

                    _ => {}
                }

                let visible_lines =
                    (state.layout.editor.height / LayoutConstants::LINE_HEIGHT) as usize;
                if let Some(ed) = state.tab_manager.active_editor_mut() {
                    ed.ensure_cursor_visible(visible_lines);
                }

                let title = state
                    .tab_manager
                    .active_editor()
                    .map(|e| e.window_title())
                    .unwrap_or_else(|| "Forge".into());
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

                let mut handled = false;
                if let Some((mx, my)) = state.last_mouse_position {
                    if let Some(sb) = &state.layout.sidebar {
                        if sb.contains(mx, my) && state.sidebar_open {
                            if self.search_panel.visible {
                                let current = self.search_panel.ui.scroll_offset as isize;
                                // Scroll direction: positive 'scroll' (from delta y) means scrolling down (view moves down, content moves up)
                                // Usually wheel down gives negative y?
                                // Wait, my scroll calculation above: -y * 3.0.
                                // If I scroll down (wheel back), y is -1. scroll is 3.0.
                                // For editor, positive scroll increases scroll_top (moves view down).
                                // So here, positive scroll should increase scroll_offset.
                                let new_val = (current + scroll as isize).max(0);
                                // Clamp to max results
                                let max_scroll = self.search_panel.results.len().saturating_sub(1);
                                let clamped = (new_val as usize).min(max_scroll);

                                self.search_panel.ui.scroll_offset = clamped;
                                handled = true;
                            }
                        }
                    }
                }

                if !handled {
                    if let Some(ed) = state.tab_manager.active_editor_mut() {
                        ed.scroll(scroll);
                    }
                }
                state.window.request_redraw();
            }

            WindowEvent::RedrawRequested => {
                Self::render(
                    &mut self.extension_host,
                    state,
                    &self.current_mode,
                    &self.theme,
                    &mut self.bottom_panel,
                    &self.find_bar,
                    &self.replace_bar,
                    &self.go_to_line,
                    &self.command_palette,
                    &mut self.search_panel,
                    &self.settings_ui,
                    &mut self.notifications,
                    &self.context_menu,
                    self.screenshot_path.as_ref(),
                );
            }

            _ => {
                Self::handle_input(
                    self.modifiers,
                    state,
                    &event,
                    self.bottom_panel.visible,
                    &mut self.context_menu,
                    &mut self.search_panel,
                    &self.rt,
                    &self.lsp_client,
                );
            }
        }
    }
}
