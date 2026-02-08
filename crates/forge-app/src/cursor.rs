use crate::rect_renderer::Rect;
use crate::ui::{colors, LayoutConstants, Zone};
use std::time::Instant;

/// Cursor rendering with blink support
pub struct CursorRenderer {
    /// Whether cursor is visible (for blinking)
    visible: bool,
    /// Last toggle time
    last_toggle: Instant,
    /// Blink interval
    blink_interval_ms: u64,
    /// Whether blinking is enabled
    blink_enabled: bool,
    /// Cursor style
    style: CursorStyle,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CursorStyle {
    /// Thin vertical line (default, like VS Code)
    Line,
    /// Full character block
    Block,
    /// Underline
    Underline,
}

impl CursorRenderer {
    pub fn new() -> Self {
        Self {
            visible: true,
            last_toggle: Instant::now(),
            blink_interval_ms: 530,
            blink_enabled: true,
            style: CursorStyle::Line,
        }
    }

    /// Update blink state (call every frame)
    pub fn update(&mut self) {
        if !self.blink_enabled {
            self.visible = true;
            return;
        }

        let elapsed = self.last_toggle.elapsed().as_millis() as u64;
        if elapsed >= self.blink_interval_ms {
            self.visible = !self.visible;
            self.last_toggle = Instant::now();
        }
    }

    /// Reset blink (make cursor visible immediately — call on any keypress)
    pub fn reset_blink(&mut self) {
        self.visible = true;
        self.last_toggle = Instant::now();
    }

    /// Set cursor style
    pub fn set_style(&mut self, style: CursorStyle) {
        self.style = style;
    }

    /// Enable/disable blinking
    pub fn set_blink_enabled(&mut self, enabled: bool) {
        self.blink_enabled = enabled;
        if !enabled {
            self.visible = true;
        }
    }

    /// Generate cursor rectangle
    /// `cursor_line` and `cursor_col` are 0-indexed
    /// `scroll_top` is the first visible line (0-indexed)
    pub fn render_rect(
        &self,
        cursor_line: usize,
        cursor_col: usize,
        scroll_top: usize,
        editor_zone: &Zone,
    ) -> Option<Rect> {
        if !self.visible {
            return None;
        }

        // Check if cursor is in visible area
        if cursor_line < scroll_top {
            return None;
        }
        let visible_line = cursor_line - scroll_top;
        let visible_lines = (editor_zone.height / LayoutConstants::LINE_HEIGHT) as usize;
        if visible_line >= visible_lines {
            return None;
        }

        let x = editor_zone.x + (cursor_col as f32 * LayoutConstants::CHAR_WIDTH);
        let y = editor_zone.y + (visible_line as f32 * LayoutConstants::LINE_HEIGHT);

        let (width, height) = match self.style {
            CursorStyle::Line => (2.0, LayoutConstants::LINE_HEIGHT),
            CursorStyle::Block => (LayoutConstants::CHAR_WIDTH, LayoutConstants::LINE_HEIGHT),
            CursorStyle::Underline => (LayoutConstants::CHAR_WIDTH, 2.0),
        };

        let adjusted_y = match self.style {
            CursorStyle::Underline => y + LayoutConstants::LINE_HEIGHT - 2.0,
            _ => y,
        };

        Some(Rect {
            x,
            y: adjusted_y,
            width,
            height,
            color: colors::CURSOR,
        })
    }

    /// Generate current line highlight rectangle
    pub fn current_line_rect(
        &self,
        cursor_line: usize,
        scroll_top: usize,
        editor_zone: &Zone,
    ) -> Option<Rect> {
        if cursor_line < scroll_top {
            return None;
        }
        let visible_line = cursor_line - scroll_top;
        let visible_lines = (editor_zone.height / LayoutConstants::LINE_HEIGHT) as usize;
        if visible_line >= visible_lines {
            return None;
        }

        let y = editor_zone.y + (visible_line as f32 * LayoutConstants::LINE_HEIGHT);

        Some(Rect {
            x: editor_zone.x,
            y,
            width: editor_zone.width,
            height: LayoutConstants::LINE_HEIGHT,
            color: colors::CURRENT_LINE,
        })
    }

    /// Generate selection rectangles
    pub fn selection_rects(
        &self,
        sel_start_line: usize,
        sel_start_col: usize,
        sel_end_line: usize,
        sel_end_col: usize,
        scroll_top: usize,
        line_lengths: &[usize], // length of each line in characters
        editor_zone: &Zone,
    ) -> Vec<Rect> {
        let mut rects = Vec::new();
        let visible_lines = (editor_zone.height / LayoutConstants::LINE_HEIGHT) as usize;

        let start_line = sel_start_line.min(sel_end_line);
        let end_line = sel_start_line.max(sel_end_line);
        let (start_col, end_col) = if sel_start_line <= sel_end_line {
            (sel_start_col, sel_end_col)
        } else {
            (sel_end_col, sel_start_col)
        let (start_col, end_col) = if sel_start_line < sel_end_line {
            (sel_start_col, sel_end_col)
        } else if sel_start_line > sel_end_line {
            (sel_end_col, sel_start_col)
        } else {
            (sel_start_col.min(sel_end_col), sel_start_col.max(sel_end_col))
        };

        for line in start_line..=end_line {
            if line < scroll_top || line >= scroll_top + visible_lines {
                continue;
            }
            let visible_line = line - scroll_top;
            let y = editor_zone.y + (visible_line as f32 * LayoutConstants::LINE_HEIGHT);

            let line_len = line_lengths.get(line).copied().unwrap_or(0);

            let col_start = if line == start_line { start_col } else { 0 };
            let col_end = if line == end_line { end_col } else { line_len };

            if col_start >= col_end && line != end_line {
                continue;
            }

            let x = editor_zone.x + (col_start as f32 * LayoutConstants::CHAR_WIDTH);
            let width = ((col_end - col_start) as f32 * LayoutConstants::CHAR_WIDTH).max(LayoutConstants::CHAR_WIDTH);

            rects.push(Rect {
                x,
                y,
                width,
                height: LayoutConstants::LINE_HEIGHT,
                color: colors::SELECTION,
            });
        }

        rects
    }
}

impl Default for CursorRenderer {
    fn default() -> Self {
        Self::new()
    }
}
