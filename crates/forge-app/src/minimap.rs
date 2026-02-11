#[derive(Debug, Clone, PartialEq)]
pub struct MinimapLine {
    pub line: usize,
    pub color: [f32; 3],
}

pub struct Minimap {
    pub lines: Vec<MinimapLine>,
    pub viewport_start: usize,
    pub viewport_end: usize,
    pub total_lines: usize,
}

impl Minimap {
    pub fn build(total_lines: usize, viewport_start: usize, viewport_end: usize) -> Self {
        // Since we don't have content, we can't generate lines with meaningful colors.
        // We'll initialize an empty vector or maybe default lines?
        // The prompt doesn't specify how `lines` are populated.
        // We'll leave it empty for the caller to fill, or fill with defaults if expected.
        // Given "build" usually constructs the object, and we have total_lines...
        // Let's just store the params.

        Self {
            lines: Vec::new(),
            viewport_start,
            viewport_end,
            total_lines,
        }
    }

    pub fn click_to_line(&self, y_fraction: f32) -> usize {
        if self.total_lines == 0 {
            return 0;
        }

        // y_fraction is 0.0 to 1.0 representing position in the minimap height.
        // If minimap shows the *whole* file, then:
        let line = (y_fraction * self.total_lines as f32).floor() as usize;

        if line >= self.total_lines {
            self.total_lines - 1
        } else {
            line
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build() {
        let mm = Minimap::build(100, 10, 20);
        assert_eq!(mm.total_lines, 100);
        assert_eq!(mm.viewport_start, 10);
        assert_eq!(mm.viewport_end, 20);
        assert!(mm.lines.is_empty());
    }

    #[test]
    fn test_click_to_line() {
        let mm = Minimap::build(100, 0, 10);

        // Top
        assert_eq!(mm.click_to_line(0.0), 0);

        // Middle
        assert_eq!(mm.click_to_line(0.5), 50);

        // Bottom (almost)
        assert_eq!(mm.click_to_line(0.99), 99);

        // Bottom (1.0 should map to last line or clamp)
        assert_eq!(mm.click_to_line(1.0), 99);

        // Out of bounds
        assert_eq!(mm.click_to_line(1.5), 99);
    }
}
