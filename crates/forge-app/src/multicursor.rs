#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection {
    pub start: Position,
    pub end: Position,
}

impl Selection {
    pub fn new(line: usize, col: usize) -> Self {
        Self {
            start: Position { line, col },
            end: Position { line, col },
        }
    }

    pub fn range(start_line: usize, start_col: usize, end_line: usize, end_col: usize) -> Self {
        Self {
            start: Position {
                line: start_line,
                col: start_col,
            },
            end: Position {
                line: end_line,
                col: end_col,
            },
        }
    }
}

pub struct MultiCursor {
    pub cursors: Vec<Selection>,
}

impl Default for MultiCursor {
    fn default() -> Self {
        Self {
            cursors: Vec::new(),
        }
    }
}

impl MultiCursor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_cursor(&mut self, line: usize, col: usize) {
        self.cursors.push(Selection::new(line, col));
    }

    pub fn clear(&mut self) {
        self.cursors.clear();
    }

    pub fn cursor_count(&self) -> usize {
        self.cursors.len()
    }

    /// Simulates Ctrl+D: adds the next occurrence of `word` as a selection
    pub fn select_next_occurrence(&mut self, text: &str, word: &str) {
        if self.cursors.is_empty() {
            // If no cursors, find first occurrence
            if let Some((line, col)) = self.find_first(text, word) {
                self.cursors
                    .push(Selection::range(line, col, line, col + word.len()));
            }
            return;
        }

        // Find the last cursor's position
        let last_cursor = self.cursors.last().unwrap();
        let start_search_line = last_cursor.end.line;
        let start_search_col = last_cursor.end.col;

        let mut found = false;

        for (i, line_str) in text.lines().enumerate().skip(start_search_line) {
            let search_start = if i == start_search_line {
                start_search_col
            } else {
                0
            };

            if search_start > line_str.len() {
                continue;
            }

            if let Some(idx) = line_str[search_start..].find(word) {
                let absolute_col = search_start + idx;
                self.cursors.push(Selection::range(
                    i,
                    absolute_col,
                    i,
                    absolute_col + word.len(),
                ));
                found = true;
                break;
            }
        }

        // Wrap around if not found? Usually Ctrl+D wraps around.
        if !found {
            if let Some((line, col)) = self.find_first(text, word) {
                // Only add if not already selected (simplistic check)
                let is_selected = self
                    .cursors
                    .iter()
                    .any(|s| s.start.line == line && s.start.col == col);
                if !is_selected {
                    self.cursors
                        .push(Selection::range(line, col, line, col + word.len()));
                }
            }
        }
    }

    pub fn select_all_occurrences(&mut self, text: &str, word: &str) {
        self.clear();
        for (i, line_str) in text.lines().enumerate() {
            let mut start = 0;
            while let Some(idx) = line_str[start..].find(word) {
                let absolute_col = start + idx;
                self.cursors.push(Selection::range(
                    i,
                    absolute_col,
                    i,
                    absolute_col + word.len(),
                ));
                start = absolute_col + word.len();
            }
        }
    }

    fn find_first(&self, text: &str, word: &str) -> Option<(usize, usize)> {
        for (i, line_str) in text.lines().enumerate() {
            if let Some(idx) = line_str.find(word) {
                return Some((i, idx));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_cursor() {
        let mut mc = MultiCursor::new();
        mc.add_cursor(1, 5);
        assert_eq!(mc.cursor_count(), 1);
        assert_eq!(mc.cursors[0].start.line, 1);
        assert_eq!(mc.cursors[0].start.col, 5);
    }

    #[test]
    fn test_select_next_occurrence() {
        let mut mc = MultiCursor::new();
        let text = "hello world\nhello universe";

        // First Ctrl+D finds first hello
        mc.select_next_occurrence(text, "hello");
        assert_eq!(mc.cursor_count(), 1);
        assert_eq!(mc.cursors[0].start.line, 0);
        assert_eq!(mc.cursors[0].start.col, 0);

        // Second Ctrl+D finds second hello
        mc.select_next_occurrence(text, "hello");
        assert_eq!(mc.cursor_count(), 2);
        assert_eq!(mc.cursors[1].start.line, 1);
        assert_eq!(mc.cursors[1].start.col, 0);
    }

    #[test]
    fn test_select_all_occurrences() {
        let mut mc = MultiCursor::new();
        let text = "foo bar foo baz foo";

        mc.select_all_occurrences(text, "foo");
        assert_eq!(mc.cursor_count(), 3);
        assert_eq!(mc.cursors[0].start.col, 0);
        assert_eq!(mc.cursors[1].start.col, 8);
        assert_eq!(mc.cursors[2].start.col, 16);
    }
}
