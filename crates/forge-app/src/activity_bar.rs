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
    Account,
    Settings,
}

impl ActivityItem {
    /// Unicode icon character for each item (using Codicons via forge-icons)
    pub fn icon_char(&self) -> &'static str {
        match self {
            Self::Explorer => forge_icons::UiIcon::Folder.glyph(),
            Self::Search => forge_icons::UiIcon::Search.glyph(),
            Self::SourceControl => forge_icons::UiIcon::SourceControl.glyph(),
            Self::Debug => forge_icons::UiIcon::Debug.glyph(),
            Self::Extensions => forge_icons::UiIcon::Extensions.glyph(),
            Self::AiAgent => forge_icons::UiIcon::AiAgent.glyph(),
            Self::Account => forge_icons::UiIcon::Account.glyph(),
            Self::Settings => forge_icons::UiIcon::Settings.glyph(),
        }
    }

    /// Label for tooltip
    #[allow(dead_code)]
    pub fn label(&self) -> &'static str {
        match self {
            Self::Explorer => "Explorer",
            Self::Search => "Search",
            Self::SourceControl => "Source Control",
            Self::Debug => "Run and Debug",
            Self::Extensions => "Extensions",
            Self::AiAgent => "AI Chat",
            Self::Account => "Accounts",
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
        &[
            ActivityItem::Account,
            ActivityItem::Settings
        ]
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
    pub fn render_rects(&self, zone: &Zone, theme: &forge_theme::Theme) -> Vec<Rect> {
        let mut rects = Vec::with_capacity(16);
        let item_size = LayoutConstants::ACTIVITY_BAR_WIDTH; // Should be square usually 48x48

        // Theme colors
        let active_border = theme
            .color("activityBar.activeBorder")
            .unwrap_or(colors::TEXT_WHITE);
        let active_bg = theme.color("activityBar.activeBackground");
        let _inactive_fg = theme
            .color("activityBar.inactiveForeground")
            .unwrap_or([1.0, 1.0, 1.0, 0.4]);

        // Top items
        for (i, item) in ActivityItem::top_items().iter().enumerate() {
            let y = zone.y + (i as f32 * item_size);
            let is_active = self.active_item == Some(*item);
            let is_hovered = self.hovered_item == Some(*item);

            if is_active {
                // Active indicator (white bar on left)
                rects.push(Rect {
                    x: zone.x,
                    y,
                    width: 2.0,
                    height: item_size,
                    color: active_border,
                });
                // Optional active background
                if let Some(bg) = active_bg {
                    rects.push(Rect {
                        x: zone.x + 2.0,
                        y,
                        width: item_size - 2.0,
                        height: item_size,
                        color: bg,
                    });
                }
            } else if is_hovered {
                // Hover background (VS Code usually handles this via opacity or slight lighten)
                // We'll use a hardcoded safe hover if not in theme (theme usually doesn't have activityBar.hoverBackground)
                rects.push(Rect {
                    x: zone.x,
                    y,
                    width: item_size,
                    height: item_size,
                    color: [1.0, 1.0, 1.0, 0.1],
                });
            }
        }

        // Bottom items
        let bottom_items = ActivityItem::bottom_items();
        for (i, item) in bottom_items.iter().enumerate() {
            let y = zone.y + zone.height - ((bottom_items.len() - i) as f32 * item_size);
            let is_active = self.active_item == Some(*item);
            let is_hovered = self.hovered_item == Some(*item);

            if is_active {
                rects.push(Rect {
                    x: zone.x,
                    y,
                    width: 2.0,
                    height: item_size,
                    color: active_border,
                });
                if let Some(bg) = active_bg {
                    rects.push(Rect {
                        x: zone.x + 2.0,
                        y,
                        width: item_size - 2.0,
                        height: item_size,
                        color: bg,
                    });
                }
            } else if is_hovered {
                rects.push(Rect {
                    x: zone.x,
                    y,
                    width: item_size,
                    height: item_size,
                    color: [1.0, 1.0, 1.0, 0.1],
                });
            }
        }

        rects
    }

    /// Get icon text positions for rendering
    /// Returns (text, x, y, color)
    #[allow(dead_code)]
    pub fn text_positions(
        &self,
        zone: &Zone,
        theme: &forge_theme::Theme,
    ) -> Vec<(&'static str, f32, f32, [f32; 4])> {
        let mut result = Vec::with_capacity(8);
        let item_size = LayoutConstants::ACTIVITY_BAR_WIDTH;

        let fg = theme
            .color("activityBar.foreground")
            .unwrap_or(colors::TEXT_WHITE);
        let inactive_fg = theme
            .color("activityBar.inactiveForeground")
            .unwrap_or([1.0, 1.0, 1.0, 0.4]);

        // Top items
        for (i, item) in ActivityItem::top_items().iter().enumerate() {
            let x = zone.x + (item_size - 24.0) / 2.0; // Centered roughly (assuming 24px font)
            let y = zone.y + (i as f32 * item_size) + (item_size - 24.0) / 2.0;

            let is_active = self.active_item == Some(*item);
            let color = if is_active { fg } else { inactive_fg };

            result.push((item.icon_char(), x, y, color));
        }

        // Bottom items
        let bottom_items = ActivityItem::bottom_items();
        for (i, item) in bottom_items.iter().enumerate() {
            let x = zone.x + (item_size - 24.0) / 2.0;
            let y = zone.y + zone.height - ((bottom_items.len() - i) as f32 * item_size)
                + (item_size - 24.0) / 2.0;

            let is_active = self.active_item == Some(*item);
            let color = if is_active { fg } else { inactive_fg };

            result.push((item.icon_char(), x, y, color));
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
