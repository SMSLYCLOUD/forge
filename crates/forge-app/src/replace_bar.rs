use crate::find_bar::Match;

pub struct ReplaceBar {
    pub visible: bool,
    pub replace_text: String,
}

impl Default for ReplaceBar {
    fn default() -> Self {
        Self {
            visible: false,
            replace_text: String::new(),
        }
    }
}

impl ReplaceBar {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(&mut self) {
        self.visible = true;
    }

    pub fn close(&mut self) {
        self.visible = false;
        self.replace_text.clear();
    }

    pub fn replace_current(
        &self,
        text: &mut String,
        _find: &str, // Not strictly needed if we trust match_pos, but good for verification
        replace: &str,
        match_pos: &Match,
    ) -> String {
        // We need to find the byte offset for the line/col.
        // This is tricky without a rope or line index.
        // Assuming `text` is the full buffer content.

        let lines: Vec<&str> = text.lines().collect();
        if match_pos.line >= lines.len() {
            return text.clone();
        }

        // Reconstruct the text with replacement
        // This is inefficient for large files but fits the signature

        // We can't easily modify `lines` in place because `&str` is immutable view.
        // We have to build a new string.

        let mut new_text = String::new();
        for (i, line) in text.lines().enumerate() {
            if i == match_pos.line {
                let mut new_line = String::new();
                if match_pos.start_col <= line.len() && match_pos.end_col <= line.len() {
                    new_line.push_str(&line[..match_pos.start_col]);
                    new_line.push_str(replace);
                    new_line.push_str(&line[match_pos.end_col..]);
                } else {
                    new_line.push_str(line);
                }
                new_text.push_str(&new_line);
            } else {
                new_text.push_str(line);
            }
            new_text.push('\n'); // standardized newline
        }

        // Remove trailing newline if original didn't have one?
        // The `lines()` iterator swallows newlines.
        // We should probably preserve the original newline style if possible, but for this task, appending `\n` is a reasonable approximation for "text editor buffer" which usually ends with newline.
        // However, `lines()` does not include the final newline if the string ends with one.
        if !text.ends_with('\n') && new_text.ends_with('\n') {
            new_text.pop();
        }

        *text = new_text.clone();
        new_text
    }

    pub fn replace_all(&self, text: &str, find: &str, replace: &str) -> (String, usize) {
        // Simple string replacement for now.
        // In a real editor, this would use the FindBar's search logic (regex/case/etc).
        // But the signature here just takes `find` string.
        // Assuming standard string replacement.

        let count = text.matches(find).count();
        let new_text = text.replace(find, replace);

        (new_text, count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::find_bar::Match;

    #[test]
    fn test_replace_current() {
        let bar = ReplaceBar::new();
        let mut text = String::from("hello world\nhello universe");
        let m = Match {
            line: 0,
            start_col: 0,
            end_col: 5,
        };

        let new_text = bar.replace_current(&mut text, "hello", "hi", &m);
        assert_eq!(new_text, "hi world\nhello universe");
        assert_eq!(text, "hi world\nhello universe");
    }

    #[test]
    fn test_replace_all() {
        let bar = ReplaceBar::new();
        let text = "hello world\nhello universe";

        let (new_text, count) = bar.replace_all(text, "hello", "hi");
        assert_eq!(count, 2);
        assert_eq!(new_text, "hi world\nhi universe");
    }
}
