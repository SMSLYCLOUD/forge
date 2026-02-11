use tree_sitter::{Tree, Node, TreeCursor};

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    Function,
    Struct,
    Enum,
    Impl,
    Class,
    Method,
    Variable,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub line: usize, // 0-based
    pub children: Vec<Symbol>,
}

#[derive(Default)]
pub struct OutlinePanel {
    pub symbols: Vec<Symbol>,
    pub visible: bool,
}

impl OutlinePanel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn extract_symbols(&mut self, tree: &Tree, source: &str) {
        self.symbols.clear();
        let cursor = tree.walk();

        // Start from root
        let root = tree.root_node();
        // We iterate children of root.
        // Actually, recursive walk needed.

        self.symbols = Self::visit_node(root, source);
    }

    fn visit_node(node: Node, source: &str) -> Vec<Symbol> {
        let mut symbols = Vec::new();
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            let kind_str = child.kind();

            let symbol_kind = match kind_str {
                "function_item" | "function_definition" => SymbolKind::Function,
                "struct_item" | "struct_definition" => SymbolKind::Struct,
                "enum_item" | "enum_definition" => SymbolKind::Enum,
                "impl_item" | "impl_definition" => SymbolKind::Impl,
                "class_definition" => SymbolKind::Class,
                "method_definition" => SymbolKind::Method,
                "let_declaration" | "const_item" => SymbolKind::Variable,
                _ => SymbolKind::Unknown,
            };

            if symbol_kind != SymbolKind::Unknown {
                // Extract name
                let name = Self::extract_name(child, source).unwrap_or_else(|| "Anonymous".to_string());

                // Recurse for children (e.g. methods inside impl)
                let children = Self::visit_node(child, source);

                symbols.push(Symbol {
                    name,
                    kind: symbol_kind,
                    line: child.start_position().row,
                    children,
                });
            } else {
                // Even if not a symbol itself, recurse to find nested symbols (e.g. mod block)
                // But avoid clutter. Usually symbols are top-level or inside impl/class.
                if kind_str == "mod_item" || kind_str == "module" {
                    let children = Self::visit_node(child, source);
                    if !children.is_empty() {
                         let name = Self::extract_name(child, source).unwrap_or_else(|| "mod".to_string());
                         symbols.push(Symbol {
                             name,
                             kind: SymbolKind::Class, // Treat mod as class/container
                             line: child.start_position().row,
                             children,
                         });
                    }
                } else if kind_str == "declaration_list" || kind_str == "impl_item" {
                    // impl_item is handled above.
                    // But declaration_list is block inside impl/class.
                    symbols.extend(Self::visit_node(child, source));
                }
            }
        }

        symbols
    }

    fn extract_name(node: Node, source: &str) -> Option<String> {
        // Look for "name" or "identifier" child
        if let Some(name_node) = node.child_by_field_name("name") {
            return Some(Self::node_text(name_node, source));
        }

        // Fallback: iterate children and find identifier
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" || child.kind() == "type_identifier" {
                return Some(Self::node_text(child, source));
            }
        }

        None
    }

    fn node_text(node: Node, source: &str) -> String {
        let start = node.start_byte();
        let end = node.end_byte();
        if start < source.len() && end <= source.len() {
            source[start..end].to_string()
        } else {
            "".to_string()
        }
    }
}
