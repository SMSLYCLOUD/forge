pub struct BracketMatcher;

impl BracketMatcher {
    pub fn find_match(text: &str, line: usize, col: usize) -> Option<(usize, usize)> {
        // Identify the bracket at the target position to ensure it is a bracket
        // This is a bit inefficient since we scan everything anyway, but good for early exit?
        // Actually, we can just run the full scan and check if we find a pair involving (line, col).

        let target_pos = (line, col);

        // Stack stores (char, (line, col))
        let mut stack: Vec<(char, (usize, usize))> = Vec::new();

        let mut current_line = 0;
        let mut current_col = 0;

        let mut chars = text.char_indices().peekable();

        let mut in_string = false;
        let mut in_line_comment = false;
        let mut in_block_comment = false;

        // Helper to check if we match the target
        let is_target = |l, c| l == target_pos.0 && c == target_pos.1;

        while let Some((_, c)) = chars.next() {
            // Handle newlines for line/col tracking
            if c == '\n' {
                current_line += 1;
                current_col = 0;
                in_line_comment = false;
                // Keep in_string state for multiline strings
                continue;
            }

            let pos = (current_line, current_col);
            current_col += 1; // Advance col for NEXT char? No, current char is at current_col.
                              // Actually, if I increment after processing, then (line, col) matches my logic.
                              // Let's assume 0-indexed cols.

            // State machine update
            if in_line_comment {
                continue;
            }

            if in_block_comment {
                if c == '*' {
                    if let Some((_, '/')) = chars.peek() {
                        chars.next(); // consume '/'
                        current_col += 1;
                        in_block_comment = false;
                    }
                }
                continue;
            }

            if in_string {
                if c == '\\' {
                    // Escape next char
                    if let Some(_) = chars.next() {
                        current_col += 1;
                    }
                } else if c == '"' {
                    in_string = false;
                }
                continue;
            }

            // Normal state
            match c {
                '/' => {
                    if let Some((_, '/')) = chars.peek() {
                        chars.next();
                        current_col += 1;
                        in_line_comment = true;
                        continue;
                    } else if let Some((_, '*')) = chars.peek() {
                        chars.next();
                        current_col += 1;
                        in_block_comment = true;
                        continue;
                    }
                }
                '"' => {
                    in_string = true;
                    continue;
                }
                '(' | '[' | '{' => {
                    stack.push((c, pos));
                }
                ')' | ']' | '}' => {
                    if let Some((open_char, open_pos)) = stack.pop() {
                        if !Self::matches(open_char, c) {
                            // Mismatched bracket, ignore or handle?
                            // For simple matching, maybe we just assume code is well-formed or we ignore mismatches.
                            // But if we popped, we consumed the open bracket.
                            // If it doesn't match, strictly speaking, it's a syntax error.
                            // But let's proceed.
                        } else {
                            // Check if this pair involves our target
                            if is_target(open_pos.0, open_pos.1) {
                                return Some(pos);
                            }
                            if is_target(pos.0, pos.1) {
                                return Some(open_pos);
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        None
    }

    fn matches(open: char, close: char) -> bool {
        match (open, close) {
            ('(', ')') => true,
            ('[', ']') => true,
            ('{', '}') => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_basic() {
        let text = "fn foo() { }";
        // ( is at 0:6
        // ) is at 0:7
        // { is at 0:9
        // } is at 0:11

        assert_eq!(BracketMatcher::find_match(text, 0, 6), Some((0, 7)));
        assert_eq!(BracketMatcher::find_match(text, 0, 7), Some((0, 6)));
        assert_eq!(BracketMatcher::find_match(text, 0, 9), Some((0, 11)));
        assert_eq!(BracketMatcher::find_match(text, 0, 11), Some((0, 9)));
    }

    #[test]
    fn test_match_nested() {
        let text = "(( ))";
        // Outer ( at 0
        // Inner ( at 1
        // Inner ) at 3
        // Outer ) at 4

        assert_eq!(BracketMatcher::find_match(text, 0, 0), Some((0, 4)));
        assert_eq!(BracketMatcher::find_match(text, 0, 1), Some((0, 3)));
    }

    #[test]
    fn test_skip_strings() {
        let text = "let s = \"(\"; )";
        // "(\" is a string. The ( inside should be ignored.
        // The last ) at 13 has no match (or matches something before?).
        // Wait, text length is 14.
        // 01234567890123
        // let s = "("; )

        // The ( inside string is at 9.
        assert_eq!(BracketMatcher::find_match(text, 0, 9), None);

        // If we had a real pair outside:
        let text2 = "( \" ) \" )";
        // 0: (
        // 2: "
        // 4: )  <- inside string
        // 6: "
        // 8: )

        // The ( at 0 matches ) at 8. ) at 4 is ignored.
        assert_eq!(BracketMatcher::find_match(text2, 0, 0), Some((0, 8)));
    }

    #[test]
    fn test_skip_comments() {
        let text = "( // ) \n )";
        // ( at 0
        // ) at 5 is in comment
        // ) at 9 (line 1, col 1) is real match

        assert_eq!(BracketMatcher::find_match(text, 0, 0), Some((1, 1)));
    }
}
