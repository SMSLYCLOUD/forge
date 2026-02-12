/// Find all references to a symbol in text
use crate::go_to_def::Location;

/// Check if a position in text is at a word boundary
fn is_word_boundary(text: &str, start: usize, end: usize) -> bool {
    let bytes = text.as_bytes();
    let before_ok =
        start == 0 || !bytes[start - 1].is_ascii_alphanumeric() && bytes[start - 1] != b'_';
    let after_ok = end >= bytes.len() || !bytes[end].is_ascii_alphanumeric() && bytes[end] != b'_';
    before_ok && after_ok
}

/// Find all whole-word references to `symbol` in text
pub fn find_references(symbol: &str, file_name: &str, text: &str) -> Vec<Location> {
    let mut results = Vec::new();
    for (line_idx, line) in text.lines().enumerate() {
        let mut search_from = 0;
        while let Some(pos) = line[search_from..].find(symbol) {
            let abs_pos = search_from + pos;
            let end_pos = abs_pos + symbol.len();
            if is_word_boundary(line, abs_pos, end_pos) {
                results.push(Location {
                    file: file_name.to_string(),
                    line: line_idx,
                    col: abs_pos,
                });
            }
            search_from = end_pos;
        }
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_references() {
        let text = "fn foo() {}\nlet x = foo();\nfoo();";
        let refs = find_references("foo", "main.rs", text);
        assert_eq!(refs.len(), 3);
    }

    #[test]
    fn test_excludes_partial() {
        let text = "foobar()\nfoo()";
        let refs = find_references("foo", "main.rs", text);
        // "foo" in "foobar" should be excluded due to word boundary
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].line, 1);
    }
}
