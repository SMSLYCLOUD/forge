use crate::rect_renderer::Rect;
use crate::ui::{colors, LayoutConstants, Zone};

/// Editor gutter — line numbers + diagnostics indicators
pub struct Gutter {
    /// First visible line (0-indexed)
    pub scroll_top: usize,
    /// Total lines in the file
    pub total_lines: usize,
    /// Current cursor line (0-indexed)
    pub cursor_line: usize,
    /// Lines with errors (line number → severity)
    pub diagnostics: Vec<(usize, DiagnosticSeverity)>,
    /// Lines with breakpoints
    pub breakpoints: Vec<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

impl Gutter {
    pub fn new() -> Self {
        Self {
            scroll_top: 0,
            total_lines: 1,
            cursor_line: 0,
            diagnostics: Vec::new(),
            breakpoints: Vec::new(),
        }
    }

    /// Calculate how many lines fit in the visible area
    pub fn visible_lines(zone: &Zone) -> usize {
        (zone.height / LayoutConstants::LINE_HEIGHT).floor() as usize
    }

    /// Generate rectangles for gutter decorations
    pub fn render_rects(&self, zone: &Zone) -> Vec<Rect> {
        let mut rects = Vec::with_capacity(32);
        let visible = Self::visible_lines(zone);

        for i in 0..visible {
            let line = self.scroll_top + i;
            if line >= self.total_lines {
                break;
            }
            let y = zone.y + (i as f32 * LayoutConstants::LINE_HEIGHT);

            // Current line highlight
            if line == self.cursor_line {
                rects.push(Rect {
                    x: zone.x,
                    y,
                    width: zone.width,
                    height: LayoutConstants::LINE_HEIGHT,
                    color: colors::CURRENT_LINE,
                });
            }

            // Breakpoint indicator (red circle area)
            if self.breakpoints.contains(&line) {
                rects.push(Rect {
                    x: zone.x + 4.0,
                    y: y + 3.0,
                    width: 14.0,
                    height: 14.0,
                    color: colors::ERROR,
                });
            }

            // Diagnostic indicator (colored bar on the right edge of gutter)
            if let Some((_, severity)) = self.diagnostics.iter().find(|(l, _)| *l == line) {
                let color = match severity {
                    DiagnosticSeverity::Error => colors::ERROR,
                    DiagnosticSeverity::Warning => colors::WARNING,
                    DiagnosticSeverity::Info => colors::STATUS_BAR,
                    DiagnosticSeverity::Hint => colors::TEXT_DIM,
                };
                rects.push(Rect {
                    x: zone.x + zone.width - 3.0,
                    y,
                    width: 3.0,
                    height: LayoutConstants::LINE_HEIGHT,
                    color,
                });
            }
        }

        rects
    }

    /// Get line number text positions (returns (text, x, y, is_current_line))
    pub fn text_positions(&self, zone: &Zone) -> Vec<(String, f32, f32, bool)> {
        let visible = Self::visible_lines(zone);
        let mut result = Vec::with_capacity(visible);
        let line_num_width = format!("{}", self.total_lines).len();

        for i in 0..visible {
            let line = self.scroll_top + i;
            if line >= self.total_lines {
                break;
            }
            let text = format!("{:>width$}", line + 1, width = line_num_width);
            let x = zone.x + 20.0; // After breakpoint area
            let y = zone.y + (i as f32 * LayoutConstants::LINE_HEIGHT) + 2.0;
            let is_current = line == self.cursor_line;
            result.push((text, x, y, is_current));
        }

        result
    }

    /// Toggle breakpoint on a line
    pub fn toggle_breakpoint(&mut self, line: usize) {
        if let Some(idx) = self.breakpoints.iter().position(|l| *l == line) {
            self.breakpoints.remove(idx);
        } else {
            self.breakpoints.push(line);
        }
    }

    /// Handle click in gutter (toggle breakpoint)
    pub fn handle_click(&mut self, click_y: f32, zone: &Zone) -> Option<usize> {
        let relative_y = click_y - zone.y;
        if relative_y < 0.0 {
            return None;
        }
        let line_index = (relative_y / LayoutConstants::LINE_HEIGHT) as usize;
        let line = self.scroll_top + line_index;
        if line < self.total_lines {
            self.toggle_breakpoint(line);
            Some(line)
        } else {
            None
        }
    }
}

impl Default for Gutter {
    fn default() -> Self {
        Self::new()
    }
}
