//! GPU-rendered text selections and highlights.
use crate::rect_renderer::Rect;
use crate::ui::LayoutConstants;

pub struct SelectionRenderer;

impl SelectionRenderer {
    /// Render selection highlight rectangles for a given set of selection ranges.
    pub fn render_selections(
        selections: &[(usize, usize, usize, usize)], // (start_line, start_col, end_line, end_col)
        scroll_top: usize,
        editor_zone: &crate::ui::Zone,
    ) -> Vec<Rect> {
        let mut rects = Vec::new();
        let line_h = LayoutConstants::LINE_HEIGHT;
        let char_w = LayoutConstants::CHAR_WIDTH;

        for &(sl, sc, el, ec) in selections {
            if el < scroll_top {
                continue;
            }
            for line in sl..=el {
                if line < scroll_top {
                    continue;
                }
                let vis_line = line - scroll_top;
                let y = editor_zone.y + (vis_line as f32 * line_h);
                if y > editor_zone.y + editor_zone.height {
                    break;
                }

                let (x_start, x_end) = if sl == el {
                    (sc as f32 * char_w, ec as f32 * char_w)
                } else if line == sl {
                    (sc as f32 * char_w, editor_zone.width)
                } else if line == el {
                    (0.0, ec as f32 * char_w)
                } else {
                    (0.0, editor_zone.width)
                };

                rects.push(Rect {
                    x: editor_zone.x + x_start,
                    y,
                    width: x_end - x_start,
                    height: line_h,
                    color: [0.0, 0.47, 0.84, 0.23], // VS Code-like blue selection
                });
            }
        }
        rects
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn single_line_selection() {
        let zone = crate::ui::Zone {
            x: 0.0,
            y: 0.0,
            width: 800.0,
            height: 600.0,
        };
        let rects = SelectionRenderer::render_selections(&[(0, 0, 0, 5)], 0, &zone);
        assert_eq!(rects.len(), 1);
    }
}
