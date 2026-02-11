#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WrappedLine {
    pub text: String,
    pub original_line: usize,
    pub is_continuation: bool,
}

pub struct WordWrapper;

impl WordWrapper {
    pub fn wrap(line: &str, max_chars: usize) -> Vec<WrappedLine> {
        let mut result = Vec::new();
        if line.is_empty() {
            result.push(WrappedLine {
                text: String::new(),
                original_line: 0, // Placeholder, usually caller sets this or we pass it
                is_continuation: false,
            });
            return result;
        }

        if max_chars == 0 {
            // Degenerate case, maybe 1 char per line?
            // Let's force at least 1.
            // Or just return as is?
            // Return chars split one by one.
            for c in line.chars() {
                result.push(WrappedLine {
                    text: c.to_string(),
                    original_line: 0,
                    is_continuation: !result.is_empty(),
                });
            }
            return result;
        }

        let mut start = 0;
        let mut current_width = 0;
        let mut last_space_idx = None;

        let chars: Vec<(usize, char)> = line.char_indices().collect();
        let mut i = 0;

        while i < chars.len() {
            let (idx, c) = chars[i];
            current_width += 1;

            if c.is_whitespace() {
                last_space_idx = Some(i);
            }

            if current_width > max_chars {
                // We need to wrap
                if let Some(space_idx) = last_space_idx {
                    // Break at space
                    // space_idx is index in `chars`
                    if space_idx >= start {
                        let end_char_idx = chars[space_idx].0; // byte index of space
                        let line_text = line[chars[start].0..end_char_idx].to_string();

                        result.push(WrappedLine {
                            text: line_text,
                            original_line: 0,
                            is_continuation: !result.is_empty(),
                        });

                        start = space_idx + 1; // Start after space
                        i = start; // Reset loop to start of next line
                        current_width = 0;
                        last_space_idx = None;
                        continue;
                    }
                }

                // No space found or space is before start (shouldn't happen if logic correct),
                // hard wrap at previous char
                // Break at i (current char starts new line)
                let end_char_idx = idx;
                let line_text = line[chars[start].0..end_char_idx].to_string();

                result.push(WrappedLine {
                    text: line_text,
                    original_line: 0,
                    is_continuation: !result.is_empty(),
                });

                start = i;
                current_width = 0;
                last_space_idx = None;
                // Don't increment i, process current char again for new line
                continue;
            }

            i += 1;
        }

        // Push remaining
        if start < chars.len() {
            let line_text = line[chars[start].0..].to_string();
            result.push(WrappedLine {
                text: line_text,
                original_line: 0,
                is_continuation: !result.is_empty(),
            });
        }

        result
    }

    pub fn wrap_all(lines: &[&str], max_chars: usize) -> Vec<WrappedLine> {
        let mut result = Vec::new();
        for (i, line) in lines.iter().enumerate() {
            let mut wrapped = Self::wrap(line, max_chars);
            for w in &mut wrapped {
                w.original_line = i;
            }
            result.extend(wrapped);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_short_line() {
        let line = "hello world";
        let wrapped = WordWrapper::wrap(line, 20);
        assert_eq!(wrapped.len(), 1);
        assert_eq!(wrapped[0].text, "hello world");
        assert!(!wrapped[0].is_continuation);
    }

    #[test]
    fn test_wrap_at_space() {
        let line = "hello world";
        let wrapped = WordWrapper::wrap(line, 5);
        // "hello" is 5 chars. " " is 6th.
        // It fits "hello" (5). Space (6) triggers wrap?
        // Logic: current_width > max_chars.
        // i=0..4 ("hello"): width 1..5.
        // i=5 (' '): width 6. 6 > 5. Wrap.
        // last_space_idx is i=5.
        // Break at space. Text: "hello".
        // Next line starts after space. "world".

        assert_eq!(wrapped.len(), 2);
        assert_eq!(wrapped[0].text, "hello");
        assert_eq!(wrapped[1].text, "world");
        assert!(wrapped[1].is_continuation);
    }

    #[test]
    fn test_hard_wrap() {
        let line = "abcdefgh";
        let wrapped = WordWrapper::wrap(line, 3);
        // abc (3). d (4) -> wrap at d.
        // "abc"
        // def (3). g (4) -> wrap at g.
        // "def"
        // gh

        assert_eq!(wrapped.len(), 3);
        assert_eq!(wrapped[0].text, "abc");
        assert_eq!(wrapped[1].text, "def");
        assert_eq!(wrapped[2].text, "gh");
    }
}
