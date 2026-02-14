use crate::rect_renderer::Rect;

pub struct FileTreeUi {
    pub scroll_offset: usize,
    pub selected_index: Option<usize>,
    pub hovered_index: Option<usize>,
}

impl FileTreeUi {
    pub fn new() -> Self {
        Self {
            scroll_offset: 0,
            selected_index: None,
            hovered_index: None,
        }
    }

    pub fn render_rects(
        &self,
        nodes: &[DisplayNode],
        zone: &crate::ui::Zone,
        header_lines: usize,
    ) -> Vec<Rect> {
        let mut rects = Vec::new();
        let line_h = 22.0;
        let header_offset = header_lines as f32 * line_h;
        for (i, _node) in nodes.iter().enumerate() {
            let y = zone.y + header_offset + (i as f32 * line_h);
            if y > zone.y + zone.height {
                break;
            }
            if Some(i) == self.hovered_index {
                rects.push(Rect {
                    x: zone.x,
                    y,
                    width: zone.width,
                    height: line_h,
                    color: [1.0, 1.0, 1.0, 0.05],
                });
            }
            if Some(i) == self.selected_index {
                rects.push(Rect {
                    x: zone.x,
                    y,
                    width: zone.width,
                    height: line_h,
                    color: [0.0, 0.47, 0.84, 0.23],
                });
            }
        }
        rects
    }
}

pub struct DisplayNode {
    pub label: String,
    pub depth: usize,
    pub is_dir: bool,
    pub expanded: bool,
}
