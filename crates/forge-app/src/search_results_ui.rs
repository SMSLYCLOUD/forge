use crate::rect_renderer::Rect;
use crate::ui::{colors, LayoutConstants, Zone};
use forge_search::SearchResult;

pub struct SearchResultsUi {
    pub scroll_offset: usize,
    pub selected_index: Option<usize>,
    pub hovered_index: Option<usize>,
}

impl SearchResultsUi {
    pub fn new() -> Self {
        Self {
            scroll_offset: 0,
            selected_index: None,
            hovered_index: None,
        }
    }

    /// Render the search results list
    pub fn render_rects(
        &self,
        results: &[SearchResult],
        zone: &Zone,
        theme: &forge_theme::Theme,
    ) -> Vec<Rect> {
        let mut rects = Vec::new();

        // Background
        rects.push(zone.to_rect(theme.color("sideBar.background").unwrap_or(colors::SIDEBAR)));

        let line_height = 24.0;
        let visible_count = (zone.height / line_height).ceil() as usize;

        for (i, _result) in results.iter().enumerate().skip(self.scroll_offset).take(visible_count) {
            let y = zone.y + ((i - self.scroll_offset) as f32 * line_height);

            // Hover highlight
            if Some(i) == self.hovered_index {
                rects.push(Rect {
                    x: zone.x,
                    y,
                    width: zone.width,
                    height: line_height,
                    color: [1.0, 1.0, 1.0, 0.05],
                });
            }

            // Selection highlight
            if Some(i) == self.selected_index {
                rects.push(Rect {
                    x: zone.x,
                    y,
                    width: zone.width,
                    height: line_height,
                    color: [0.0, 0.47, 0.84, 0.4],
                });
            }
        }

        rects
    }

    /// Get text positions for rendering
    pub fn text_positions(
        &self,
        results: &[SearchResult],
        zone: &Zone,
        theme: &forge_theme::Theme,
    ) -> Vec<(String, f32, f32, [f32; 4])> {
        let mut texts = Vec::new();
        let line_height = 24.0;
        let visible_count = (zone.height / line_height).ceil() as usize;

        let file_color = theme.color("sideBar.foreground").unwrap_or(colors::TEXT_FG);
        let match_color = theme.color("editor.foreground").unwrap_or(colors::TEXT_FG);
        let dim_color = theme.color("descriptionForeground").unwrap_or(colors::TEXT_DIM);

        for (i, result) in results.iter().enumerate().skip(self.scroll_offset).take(visible_count) {
            let y = zone.y + ((i - self.scroll_offset) as f32 * line_height);
            let text_y = y + 5.0; // Centered vertically roughly

            // Format: "filename (line:col): text"
            // Simplified rendering for now: just one line per result

            // 1. Filename (basename)
            let path = std::path::Path::new(&result.file);
            let filename = path.file_name().map(|s| s.to_string_lossy()).unwrap_or_default();

            texts.push((filename.to_string(), zone.x + 10.0, text_y, file_color));

            // 2. Line number
            let line_info = format!(":{}", result.line);
            let filename_width = filename.len() as f32 * LayoutConstants::CHAR_WIDTH; // approx
            let line_info_width = line_info.len() as f32 * LayoutConstants::CHAR_WIDTH;
            texts.push((line_info, zone.x + 10.0 + filename_width, text_y, dim_color));

            // 3. Match text preview (truncated)
            let preview_x = zone.x + 10.0 + filename_width + line_info_width + 10.0;
            if preview_x < zone.x + zone.width {
                let max_len = 40;
                let preview = if result.text.len() > max_len {
                    format!("{}...", &result.text[0..max_len])
                } else {
                    result.text.clone()
                };
                texts.push((preview, preview_x, text_y, match_color));
            }
        }

        texts
    }

    pub fn handle_click(&mut self, my: f32, zone: &Zone) -> Option<usize> {
        if my < zone.y || my > zone.y + zone.height {
            return None;
        }

        let relative_y = my - zone.y;
        let line_height = 24.0;
        let index = self.scroll_offset + (relative_y / line_height) as usize;

        self.selected_index = Some(index);
        Some(index)
    }
}

impl Default for SearchResultsUi {
    fn default() -> Self {
        Self::new()
    }
}
