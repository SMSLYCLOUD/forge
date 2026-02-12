/// Go-to-definition navigation
///
/// Provides symbol-based navigation with a navigation stack for back/forward.

#[derive(Debug, Clone)]
pub struct Location {
    pub file: String,
    pub line: usize,
    pub col: usize,
}

pub struct NavStack {
    history: Vec<Location>,
    position: usize,
}

impl NavStack {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            position: 0,
        }
    }

    pub fn push(&mut self, loc: Location) {
        self.history.truncate(self.position);
        self.history.push(loc);
        self.position = self.history.len();
    }

    pub fn back(&mut self) -> Option<&Location> {
        if self.position > 0 {
            self.position -= 1;
            self.history.get(self.position)
        } else {
            None
        }
    }

    pub fn forward(&mut self) -> Option<&Location> {
        if self.position < self.history.len() {
            self.position += 1;
            self.history.get(self.position.saturating_sub(1))
        } else {
            None
        }
    }
}

/// Find definition of a symbol by searching workspace files for `fn`, `struct`, `enum`, etc.
pub fn find_definition(symbol: &str, workspace_files: &[(String, String)]) -> Option<Location> {
    let patterns = [
        format!("fn {}", symbol),
        format!("struct {}", symbol),
        format!("enum {}", symbol),
        format!("trait {}", symbol),
        format!("type {}", symbol),
        format!("const {}", symbol),
        format!("static {}", symbol),
        format!("mod {}", symbol),
        format!("class {}", symbol),
        format!("def {}", symbol),
    ];

    for (filename, content) in workspace_files {
        for (line_idx, line) in content.lines().enumerate() {
            for pat in &patterns {
                if let Some(col) = line.find(pat.as_str()) {
                    // Verify word boundary
                    let end = col + pat.len();
                    let at_end = end >= line.len() || !line.as_bytes()[end].is_ascii_alphanumeric();
                    if at_end {
                        return Some(Location {
                            file: filename.clone(),
                            line: line_idx,
                            col,
                        });
                    }
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_definition() {
        let files = vec![
            (
                "main.rs".to_string(),
                "fn main() {\n    println!(\"hello\");\n}".to_string(),
            ),
            (
                "lib.rs".to_string(),
                "pub struct Foo {\n    bar: i32,\n}".to_string(),
            ),
        ];
        let loc = find_definition("main", &files);
        assert!(loc.is_some());
        let loc = loc.unwrap();
        assert_eq!(loc.file, "main.rs");
        assert_eq!(loc.line, 0);
    }

    #[test]
    fn test_nav_stack() {
        let mut nav = NavStack::new();
        nav.push(Location {
            file: "a.rs".into(),
            line: 0,
            col: 0,
        });
        nav.push(Location {
            file: "b.rs".into(),
            line: 5,
            col: 0,
        });
        let back = nav.back();
        assert!(back.is_some());
        assert_eq!(back.unwrap().file, "a.rs");
    }

    #[test]
    fn test_not_found() {
        let files = vec![("a.rs".to_string(), "let x = 1;".to_string())];
        assert!(find_definition("nonexistent", &files).is_none());
    }
}
