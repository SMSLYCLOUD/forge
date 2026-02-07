/// Viewport represents the visible region of the text buffer
#[derive(Debug, Clone, Copy)]
pub struct Viewport {
    /// First visible line (0-indexed)
    pub start_line: usize,
    /// Last visible line (0-indexed, inclusive)
    pub end_line: usize,
    /// Horizontal scroll offset in characters
    pub scroll_x: f32,
    /// Vertical scroll offset in lines
    pub scroll_y: f32,
}

impl Viewport {
    pub fn new(start_line: usize, end_line: usize) -> Self {
        Self {
            start_line,
            end_line,
            scroll_x: 0.0,
            scroll_y: 0.0,
        }
    }

    /// Calculate viewport from window height and font metrics
    pub fn from_window_height(window_height: u32, line_height: f32, scroll_y: f32) -> Self {
        let visible_lines = (window_height as f32 / line_height).ceil() as usize;
        let start_line = scroll_y as usize;
        let end_line = start_line + visible_lines;

        Self {
            start_line,
            end_line,
            scroll_x: 0.0,
            scroll_y,
        }
    }

    /// Get the number of visible lines
    pub fn visible_lines(&self) -> usize {
        self.end_line.saturating_sub(self.start_line)
    }
}

impl Default for Viewport {
    fn default() -> Self {
        Self::new(0, 50) // Show first 50 lines by default
    }
}
