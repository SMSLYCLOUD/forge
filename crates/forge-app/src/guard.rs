/// Safe wrapper for fallible operations in the render path
pub struct Guard;

impl Guard {
    /// Safely get a line from the buffer, returning empty string if out of bounds
    pub fn get_line(buffer: &ropey::Rope, line_idx: usize) -> String {
        if line_idx < buffer.len_lines() {
            buffer.line(line_idx).to_string()
        } else {
            String::new()
        }
    }

    /// Clamp a value between min and max
    pub fn clamp_usize(value: usize, min: usize, max: usize) -> usize {
        value.max(min).min(max)
    }

    /// Safe division that returns default on zero divisor
    pub fn safe_div_f32(numerator: f32, denominator: f32, default: f32) -> f32 {
        if denominator.abs() < f32::EPSILON {
            default
        } else {
            numerator / denominator
        }
    }

    /// Clamp cursor to valid buffer position
    pub fn clamp_cursor(line: usize, col: usize, total_lines: usize, line_len: usize) -> (usize, usize) {
        let safe_line = if total_lines == 0 { 0 } else { line.min(total_lines - 1) };
        let safe_col = col.min(line_len);
        (safe_line, safe_col)
    }

    /// Safe slice of a string
    pub fn safe_substr(s: &str, start: usize, max_len: usize) -> &str {
        if start >= s.len() {
            ""
        } else {
            let end = (start + max_len).min(s.len());
            // Find valid char boundaries
            let start = s.floor_char_boundary(start);
            let end = s.ceil_char_boundary(end.min(s.len()));
            &s[start..end]
        }
    }
}
