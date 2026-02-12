/// Workspace-wide symbol index
///
/// Indexes all symbols across workspace files for quick fuzzy lookup.
use crate::outline_panel::SymbolKind;

#[derive(Debug, Clone)]
pub struct WorkspaceSymbol {
    pub name: String,
    pub kind: SymbolKind,
    pub file: String,
    pub line: usize,
}

pub struct WorkspaceSymbolIndex {
    pub symbols: Vec<WorkspaceSymbol>,
}

impl WorkspaceSymbolIndex {
    pub fn new() -> Self {
        Self {
            symbols: Vec::new(),
        }
    }

    /// Build index from workspace files: Vec of (filename, content)
    pub fn build(files: &[(String, String)]) -> Self {
        let mut symbols = Vec::new();
        let patterns: &[(&str, SymbolKind)] = &[
            ("fn ", SymbolKind::Function),
            ("struct ", SymbolKind::Struct),
            ("enum ", SymbolKind::Enum),
            ("trait ", SymbolKind::Class),
            ("impl ", SymbolKind::Impl),
            ("const ", SymbolKind::Variable),
            ("mod ", SymbolKind::Class),
        ];

        for (filename, content) in files {
            for (line_idx, line) in content.lines().enumerate() {
                let trimmed = line.trim();
                for (prefix, kind) in patterns {
                    if let Some(rest) = trimmed.strip_prefix("pub ") {
                        if let Some(name_part) = rest.strip_prefix(prefix) {
                            if let Some(name) = extract_identifier(name_part) {
                                symbols.push(WorkspaceSymbol {
                                    name,
                                    kind: kind.clone(),
                                    file: filename.clone(),
                                    line: line_idx,
                                });
                            }
                        }
                    } else if let Some(name_part) = trimmed.strip_prefix(prefix) {
                        if let Some(name) = extract_identifier(name_part) {
                            symbols.push(WorkspaceSymbol {
                                name,
                                kind: kind.clone(),
                                file: filename.clone(),
                                line: line_idx,
                            });
                        }
                    }
                }
            }
        }
        Self { symbols }
    }

    /// Case-insensitive search for symbols matching query
    pub fn search(&self, query: &str) -> Vec<&WorkspaceSymbol> {
        let q = query.to_lowercase();
        self.symbols
            .iter()
            .filter(|s| s.name.to_lowercase().contains(&q))
            .collect()
    }
}

fn extract_identifier(s: &str) -> Option<String> {
    let name: String = s
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_index() {
        let files = vec![(
            "main.rs".to_string(),
            "fn main() {}\npub struct Foo {}".to_string(),
        )];
        let idx = WorkspaceSymbolIndex::build(&files);
        assert!(idx.symbols.len() >= 2);
    }

    #[test]
    fn test_search() {
        let files = vec![(
            "lib.rs".to_string(),
            "pub fn hello_world() {}\nfn secret() {}".to_string(),
        )];
        let idx = WorkspaceSymbolIndex::build(&files);
        let results = idx.search("hello");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "hello_world");
    }
}
