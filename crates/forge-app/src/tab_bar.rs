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
