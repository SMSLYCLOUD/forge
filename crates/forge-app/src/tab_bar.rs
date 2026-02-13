use crate::rect_renderer::Rect;
use crate::ui::{colors, LayoutConstants, Zone};

/// Represents a single editor tab
#[derive(Clone, Debug)]
pub struct Tab {
    pub title: String,
    pub file_path: Option<String>,
    #[allow(dead_code)]
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
            if let Some(idx) = self
                .tabs
                .iter()
                .position(|t| t.file_path.as_ref() == Some(path))
            {
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    pub fn set_modified(&mut self, index: usize, modified: bool) {
        if index < self.tabs.len() {
            self.tabs[index].is_modified = modified;
        }
    }

    /// Generate rectangles for the tab bar
    pub fn render_rects(&self, zone: &Zone, theme: &forge_theme::Theme) -> Vec<Rect> {
        let mut rects = Vec::with_capacity(self.tabs.len() * 4);
        let tab_width = LayoutConstants::TAB_WIDTH;
        let tab_height = zone.height;

        let active_bg = theme
            .color("tab.activeBackground")
            .unwrap_or(colors::TAB_ACTIVE);
        let inactive_bg = theme
            .color("tab.inactiveBackground")
            .unwrap_or(colors::TAB_INACTIVE);
        let active_border_top = theme.color("tab.activeBorderTop");
        let active_border_bottom = theme.color("tab.activeBorder");
        let border_col = theme.color("tab.border").unwrap_or(colors::SEPARATOR);
        let contrast_border = theme.color("contrastBorder");

        for (i, tab) in self.tabs.iter().enumerate() {
            let x = zone.x + (i as f32 * tab_width) - self.scroll_offset;

            // Skip tabs scrolled out of view
            if x + tab_width < zone.x || x > zone.x + zone.width {
                continue;
            }

            let bg_color = if tab.is_active {
                active_bg
            } else {
                inactive_bg
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
                if let Some(col) = active_border_top {
                    rects.push(Rect {
                        x,
                        y: zone.y,
                        width: tab_width,
                        height: 2.0,
                        color: col,
                    });
                }
                if let Some(col) = active_border_bottom {
                    // Usually at bottom
                    rects.push(Rect {
                        x,
                        y: zone.y + tab_height - 2.0,
                        width: tab_width,
                        height: 2.0,
                        color: col,
                    });
                }
            }

            // Separator/Border between tabs
            // VS Code typically draws a border on the right of each tab, or all around depending on theme.
            // "tab.border" is "Border to separate tabs from each other."
            rects.push(Rect {
                x: x + tab_width - 1.0,
                y: zone.y + 4.0, // small padding
                width: 1.0,
                height: tab_height - 8.0,
                color: border_col,
            });

            // High contrast border if set
            if let Some(cb) = contrast_border {
                rects.push(Rect {
                    x,
                    y: zone.y,
                    width: tab_width,
                    height: tab_height,
                    color: cb, // This would fill it, but usually high contrast is an outline.
                               // For simplicity we ignore full outline implementation here as `Rect` is solid.
                });
            }
        }

        rects
    }

    /// Get tab titles for text rendering (returns (text, x, y, color, is_active, is_modified) tuples)
    #[allow(dead_code)]
    pub fn text_positions(
        &self,
        zone: &Zone,
        theme: &forge_theme::Theme,
    ) -> Vec<(String, f32, f32, [f32; 4], bool, bool)> {
        let mut result = Vec::with_capacity(self.tabs.len());
        let tab_width = LayoutConstants::TAB_WIDTH;
        let text_y = zone.y + (zone.height - LayoutConstants::SMALL_FONT_SIZE) / 2.0;

        let active_fg = theme
            .color("tab.activeForeground")
            .unwrap_or(colors::TEXT_WHITE);
        let inactive_fg = theme
            .color("tab.inactiveForeground")
            .unwrap_or(colors::TEXT_DIM);

        for (i, tab) in self.tabs.iter().enumerate() {
            let x = zone.x + (i as f32 * tab_width) - self.scroll_offset + 12.0;
            let title = if tab.is_modified {
                format!("● {}", tab.title)
            } else {
                tab.title.clone()
            };

            let color = if tab.is_active {
                active_fg
            } else {
                inactive_fg
            };

            result.push((title, x, text_y, color, tab.is_active, tab.is_modified));
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
