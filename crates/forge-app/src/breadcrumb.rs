use crate::rect_renderer::Rect;
use crate::ui::{colors, LayoutConstants, Zone};

/// A single breadcrumb segment
#[derive(Clone, Debug)]
pub struct BreadcrumbSegment {
    pub text: String,
    #[allow(dead_code)]
    pub kind: SegmentKind,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(dead_code)]
pub enum SegmentKind {
    Folder,
    File,
    Symbol,
}

/// Breadcrumb bar state and rendering
pub struct BreadcrumbBar {
    pub segments: Vec<BreadcrumbSegment>,
}

impl BreadcrumbBar {
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    /// Update breadcrumbs from file path and cursor position
    pub fn update_from_path(&mut self, file_path: &str) {
        self.segments.clear();

        // Split path into components
        let parts: Vec<&str> = file_path.split(['/', '\\']).collect();

        for (i, part) in parts.iter().enumerate() {
            if part.is_empty() {
                continue;
            }
            let kind = if i == parts.len() - 1 {
                SegmentKind::File
            } else {
                SegmentKind::Folder
            };
            self.segments.push(BreadcrumbSegment {
                text: part.to_string(),
                kind,
            });
        }
    }

    /// Add a symbol segment (e.g., current function name)
    #[allow(dead_code)]
    pub fn set_symbol(&mut self, symbol: Option<String>) {
        // Remove existing symbol segments
        self.segments.retain(|s| s.kind != SegmentKind::Symbol);

        if let Some(sym) = symbol {
            self.segments.push(BreadcrumbSegment {
                text: sym,
                kind: SegmentKind::Symbol,
            });
        }
    }

    /// Render separator rectangles (small chevrons between segments)
    pub fn render_rects(&self, zone: &Zone) -> Vec<Rect> {
        // Breadcrumb bar is mostly text; separators are rendered as small rects
        let mut rects = Vec::new();
        let char_width = LayoutConstants::CHAR_WIDTH;
        let padding = 6.0;
        let text_y = zone.y + (zone.height - LayoutConstants::SMALL_FONT_SIZE) / 2.0;

        let mut x = zone.x + padding;
        for (i, segment) in self.segments.iter().enumerate() {
            x += segment.text.len() as f32 * char_width + padding;

            // Separator chevron (small triangle/rect)
            if i < self.segments.len() - 1 {
                rects.push(Rect {
                    x: x + 2.0,
                    y: text_y + 2.0,
                    width: 6.0,
                    height: LayoutConstants::SMALL_FONT_SIZE - 4.0,
                    color: colors::TEXT_DIM,
                });
                x += 14.0; // Space for separator
            }
        }

        rects
    }

    /// Get text positions for rendering
    /// Returns (text, x, y, color) tuples
    #[allow(dead_code)]
    pub fn text_positions(&self, zone: &Zone) -> Vec<(String, f32, f32, [f32; 4])> {
        let mut result = Vec::with_capacity(self.segments.len() * 2);
        let char_width = LayoutConstants::CHAR_WIDTH;
        let padding = 6.0;
        let text_y = zone.y + (zone.height - LayoutConstants::SMALL_FONT_SIZE) / 2.0;

        let mut x = zone.x + padding;
        for (i, segment) in self.segments.iter().enumerate() {
            let color = match segment.kind {
                SegmentKind::Folder => colors::TEXT_DIM,
                SegmentKind::File => colors::TEXT_FG,
                SegmentKind::Symbol => colors::TEXT_FG,
            };
            result.push((segment.text.clone(), x, text_y, color));
            x += segment.text.len() as f32 * char_width + padding;

            // Separator text
            if i < self.segments.len() - 1 {
                result.push((String::from(">"), x + 2.0, text_y, colors::TEXT_DIM));
                x += 14.0;
            }
        }

        result
    }
}

impl Default for BreadcrumbBar {
    fn default() -> Self {
        Self::new()
    }
}
