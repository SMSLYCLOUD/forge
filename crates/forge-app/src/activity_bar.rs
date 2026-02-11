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
            let y = zone.y + zone.height - ((bottom_items.len() - i) as f32 * item_size)
                + item_size / 2.0
                - 8.0;
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
