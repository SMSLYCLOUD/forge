use crate::rect_renderer::Rect;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SidebarMode {
    Explorer,
    Search,
    Debug,
    Extensions,
}

impl Default for SidebarMode {
    fn default() -> Self {
        Self::Explorer
    }
}

/// VS Code color scheme — dark theme (Fallback)
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
    #[allow(dead_code)]
    pub const TEXT_FG: [f32; 4] = [0.8, 0.8, 0.8, 1.0];
    /// Dimmed text (#858585)
    pub const TEXT_DIM: [f32; 4] = [0.522, 0.522, 0.522, 1.0];
    /// White text
    pub const TEXT_WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
    /// Cursor color (#aeafad)
    pub const CURSOR: [f32; 4] = [0.682, 0.686, 0.678, 1.0];
    /// Selection color (#264f78)
    #[allow(dead_code)]
    pub const SELECTION: [f32; 4] = [0.149, 0.31, 0.471, 0.5];
    /// Error red
    pub const ERROR: [f32; 4] = [0.937, 0.325, 0.314, 1.0];
    /// Warning yellow
    pub const WARNING: [f32; 4] = [0.804, 0.682, 0.263, 1.0];
    /// Success green
    #[allow(dead_code)]
    pub const SUCCESS: [f32; 4] = [0.345, 0.663, 0.369, 1.0];
}

/// Pixel dimensions for each UI zone (VS Code Standard)
pub struct LayoutConstants;

impl LayoutConstants {
    pub const ACTIVITY_BAR_WIDTH: f32 = 48.0; // VS Code: 48px
    pub const TAB_BAR_HEIGHT: f32 = 35.0; // VS Code: 35px
    pub const BREADCRUMB_HEIGHT: f32 = 22.0; // VS Code: 22px
    pub const STATUS_BAR_HEIGHT: f32 = 22.0; // VS Code: 22px
    pub const GUTTER_WIDTH: f32 = 60.0; // Adjustable but 50-60 is standard
    pub const SIDEBAR_WIDTH: f32 = 250.0; // VS Code default is wider
    pub const SCROLLBAR_WIDTH: f32 = 14.0; // VS Code: 14px
    pub const TAB_WIDTH: f32 = 160.0; // Standard tab width
    #[allow(dead_code)]
    pub const TAB_CLOSE_SIZE: f32 = 16.0;
    pub const AI_PANEL_WIDTH: f32 = 350.0; // Copilot chat style
    pub const SEPARATOR_SIZE: f32 = 1.0;
    pub const LINE_HEIGHT: f32 = 20.0; // 1.4-1.5em for 14px font
    pub const CHAR_WIDTH: f32 = 8.4;
    pub const FONT_SIZE: f32 = 14.0; // Editor font size
    pub const SMALL_FONT_SIZE: f32 = 11.0; // UI font size
}

/// Computed layout zones (recalculated on resize)
#[derive(Clone, Debug)]
pub struct LayoutZones {
    #[allow(dead_code)]
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
    pub bottom_panel: Option<Zone>,
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
        Self {
            x,
            y,
            width,
            height,
        }
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
        bottom_panel_visible: bool,
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

        let content_x = activity_w
            + if sidebar_open {
                LayoutConstants::SIDEBAR_WIDTH
            } else {
                0.0
            };
        let ai_panel_w = if ai_panel_open {
            LayoutConstants::AI_PANEL_WIDTH
        } else {
            0.0
        };
        let content_w = (window_width - content_x - ai_panel_w).max(100.0);

        let tab_y = 0.0;
        let breadcrumb_y = LayoutConstants::TAB_BAR_HEIGHT;
        let editor_y = breadcrumb_y + LayoutConstants::BREADCRUMB_HEIGHT;

        let status_y = window_height - LayoutConstants::STATUS_BAR_HEIGHT;
        let available_h = status_y - editor_y;

        let bottom_panel_h = if bottom_panel_visible { 200.0 } else { 0.0 };
        let editor_h = available_h - bottom_panel_h;

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

        let bottom_panel = if bottom_panel_visible {
            Some(Zone::new(
                content_x,
                editor_y + editor_h,
                content_w,
                bottom_panel_h,
            ))
        } else {
            None
        };

        Self {
            window_width,
            window_height,
            activity_bar: Zone::new(
                activity_x,
                0.0,
                activity_w,
                window_height - LayoutConstants::STATUS_BAR_HEIGHT,
            ),
            sidebar,
            tab_bar: Zone::new(content_x, tab_y, content_w, LayoutConstants::TAB_BAR_HEIGHT),
            breadcrumb_bar: Zone::new(
                content_x,
                breadcrumb_y,
                content_w,
                LayoutConstants::BREADCRUMB_HEIGHT,
            ),
            gutter: Zone::new(content_x, editor_y, gutter_w, editor_h),
            editor: Zone::new(content_x + gutter_w, editor_y, editor_text_w, editor_h),
            status_bar: Zone::new(
                0.0,
                status_y,
                window_width,
                LayoutConstants::STATUS_BAR_HEIGHT,
            ),
            ai_panel,
            bottom_panel,
            scrollbar_v: Zone::new(
                content_x + gutter_w + editor_text_w,
                editor_y,
                scrollbar_w,
                editor_h,
            ),
        }
    }

    /// Generate all background rectangles for the UI chrome
    pub fn background_rects(&self, theme: &forge_theme::Theme) -> Vec<Rect> {
        let mut rects = Vec::with_capacity(32);

        let activity_bar_bg = theme
            .color("activityBar.background")
            .unwrap_or(colors::ACTIVITY_BAR);
        let activity_bar_border = theme
            .color("activityBar.border")
            .unwrap_or(colors::SEPARATOR);

        let sidebar_bg = theme.color("sideBar.background").unwrap_or(colors::SIDEBAR);
        let sidebar_border = theme.color("sideBar.border").unwrap_or(colors::SEPARATOR);

        let tab_bar_bg = theme
            .color("editorGroupHeader.tabsBackground")
            .unwrap_or(colors::TAB_BAR);
        let tab_border = theme
            .color("editorGroup.border")
            .unwrap_or(colors::SEPARATOR);

        let breadcrumb_bg = theme
            .color("breadcrumb.background")
            .unwrap_or(colors::BREADCRUMB);

        let gutter_bg = theme
            .color("editor.background") // Gutter matches editor bg in VS Code
            .unwrap_or(colors::GUTTER);

        let editor_bg = theme
            .color("editor.background")
            .unwrap_or(colors::EDITOR_BG);

        let status_bar_bg = theme
            .color("statusBar.background")
            .unwrap_or(colors::STATUS_BAR);
        let status_bar_border = theme.color("statusBar.border").unwrap_or(colors::SEPARATOR);

        let _border_col = theme.color("contrastBorder").unwrap_or(colors::SEPARATOR);

        // ─── Activity Bar ───
        rects.push(self.activity_bar.to_rect(activity_bar_bg));
        // Right border of activity bar
        if activity_bar_border[3] > 0.0 {
            rects.push(Rect {
                x: self.activity_bar.x + self.activity_bar.width - LayoutConstants::SEPARATOR_SIZE,
                y: self.activity_bar.y,
                width: LayoutConstants::SEPARATOR_SIZE,
                height: self.activity_bar.height,
                color: activity_bar_border,
            });
        }

        // ─── Sidebar (if open) ───
        if let Some(ref sb) = self.sidebar {
            rects.push(sb.to_rect(sidebar_bg));
            // Right border of sidebar
            if sidebar_border[3] > 0.0 {
                rects.push(Rect {
                    x: sb.x + sb.width,
                    y: 0.0,
                    width: LayoutConstants::SEPARATOR_SIZE,
                    height: self.window_height - LayoutConstants::STATUS_BAR_HEIGHT,
                    color: sidebar_border,
                });
            }
        }

        // ─── Tab Bar ───
        rects.push(self.tab_bar.to_rect(tab_bar_bg));
        // Bottom border of tab bar (editorGroup.border or contrastBorder)
        // In VS Code, tab bar often doesn't have a bottom border, but the active tab might cover it.
        // We'll skip a global bottom border for now to avoid clipping active tab, or make it subtle.

        // ─── Breadcrumb Bar ───
        rects.push(self.breadcrumb_bar.to_rect(breadcrumb_bg));

        // ─── Gutter ───
        rects.push(self.gutter.to_rect(gutter_bg));

        // ─── Editor Background ───
        rects.push(self.editor.to_rect(editor_bg));

        // ─── Scrollbar Track ───
        // VS Code scrollbar track is usually transparent or editor background
        rects.push(self.scrollbar_v.to_rect(editor_bg));
        // Optionally render scrollbar shadow/border if needed

        // ─── Status Bar ───
        rects.push(self.status_bar.to_rect(status_bar_bg));
        // Top border of status bar
        if status_bar_border[3] > 0.0 {
            rects.push(Rect {
                x: self.status_bar.x,
                y: self.status_bar.y,
                width: self.status_bar.width,
                height: LayoutConstants::SEPARATOR_SIZE,
                color: status_bar_border,
            });
        }

        // ─── AI Panel (if open) ───
        if let Some(ref ai) = self.ai_panel {
            rects.push(ai.to_rect(sidebar_bg));
            // Left border of AI panel
            rects.push(Rect {
                x: ai.x - LayoutConstants::SEPARATOR_SIZE,
                y: 0.0,
                width: LayoutConstants::SEPARATOR_SIZE,
                height: self.window_height - LayoutConstants::STATUS_BAR_HEIGHT,
                color: sidebar_border,
            });
        }

        // ─── Bottom Panel (if open) ───
        if let Some(ref bp) = self.bottom_panel {
            let panel_bg = theme.color("panel.background").unwrap_or(colors::EDITOR_BG);
            let panel_border = theme.color("panel.border").unwrap_or(colors::SEPARATOR);
            rects.push(bp.to_rect(panel_bg));
            // Top border of bottom panel
            rects.push(Rect {
                x: bp.x,
                y: bp.y,
                width: bp.width,
                height: LayoutConstants::SEPARATOR_SIZE,
                color: panel_border,
            });
        }

        rects
    }
}
