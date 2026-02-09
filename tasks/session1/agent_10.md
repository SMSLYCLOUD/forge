# Agent 10 — Line Decorations Framework + Selection Rendering

> **Read `tasks/GLOBAL_RULES.md` first.**

## Task A: Decoration API

### `crates/forge-renderer/src/decorations.rs`
```rust
//! Line decoration framework: underlines, highlights, inline annotations.

#[derive(Debug, Clone)]
pub enum Decoration {
    Underline { line: usize, start_col: usize, end_col: usize, color: [u8; 4], style: UnderlineStyle },
    LineBackground { line: usize, color: [u8; 4] },
    InlineText { line: usize, col: usize, text: String, color: [u8; 4] },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnderlineStyle { Solid, Wavy, Dashed, Dotted }

#[derive(Debug, Default)]
pub struct DecorationLayer {
    decorations: Vec<Decoration>,
}

impl DecorationLayer {
    pub fn new() -> Self { Self::default() }

    pub fn add(&mut self, dec: Decoration) { self.decorations.push(dec); }
    pub fn clear(&mut self) { self.decorations.clear(); }

    pub fn get_line_decorations(&self, line: usize) -> Vec<&Decoration> {
        self.decorations.iter().filter(|d| match d {
            Decoration::Underline { line: l, .. } => *l == line,
            Decoration::LineBackground { line: l, .. } => *l == line,
            Decoration::InlineText { line: l, .. } => *l == line,
        }).collect()
    }

    pub fn count(&self) -> usize { self.decorations.len() }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn add_and_query() {
        let mut layer = DecorationLayer::new();
        layer.add(Decoration::LineBackground { line: 5, color: [255, 0, 0, 50] });
        layer.add(Decoration::LineBackground { line: 10, color: [0, 255, 0, 50] });
        assert_eq!(layer.get_line_decorations(5).len(), 1);
        assert_eq!(layer.get_line_decorations(7).len(), 0);
    }
    #[test] fn clear_works() {
        let mut layer = DecorationLayer::new();
        layer.add(Decoration::LineBackground { line: 1, color: [0, 0, 0, 0] });
        assert_eq!(layer.count(), 1);
        layer.clear();
        assert_eq!(layer.count(), 0);
    }
}
```

Add `pub mod decorations;` to `crates/forge-renderer/src/lib.rs`.

## Task B: Selection Rendering

### `crates/forge-app/src/selection_render.rs`
```rust
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
        let char_w = LayoutConstants::FONT_SIZE * 0.6; // approximate monospace char width

        for &(sl, sc, el, ec) in selections {
            if el < scroll_top { continue; }
            for line in sl..=el {
                if line < scroll_top { continue; }
                let vis_line = line - scroll_top;
                let y = editor_zone.y + (vis_line as f32 * line_h);
                if y > editor_zone.y + editor_zone.height { break; }

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
                    w: x_end - x_start,
                    h: line_h,
                    color: [0, 120, 215, 60], // VS Code-like blue selection
                });
            }
        }
        rects
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn single_line_selection() {
        let zone = crate::ui::Zone { x: 0.0, y: 0.0, width: 800.0, height: 600.0 };
        let rects = SelectionRenderer::render_selections(&[(0, 0, 0, 5)], 0, &zone);
        assert_eq!(rects.len(), 1);
    }
}
```

Add `mod selection_render;` to `main.rs`.

**Acceptance**: `cargo test -p forge-renderer -p forge-app` passes.
