use crate::language::Language;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenType {
    Keyword, Function, Type, String, Number, Comment, Operator,
    Punctuation, Variable, Constant, Namespace, Property, Parameter,
    Macro, Attribute, Label, Builtin, Plain,
}

#[derive(Debug, Clone)]
pub struct HighlightSpan {
    pub start_byte: usize,
    pub end_byte: usize,
    pub token_type: TokenType,
}

pub struct Highlighter;

impl Highlighter {
    /// Walk the tree-sitter CST and classify nodes by token type.
    pub fn highlight(tree: &tree_sitter::Tree, source: &[u8], lang: Language) -> Vec<HighlightSpan> {
        let mut spans = Vec::new();
        let mut cursor = tree.walk();
        Self::walk_node(&mut cursor, source, lang, &mut spans);
        spans.sort_by_key(|s| s.start_byte);
        spans
    }

    fn walk_node(
        cursor: &mut tree_sitter::TreeCursor,
        source: &[u8],
        lang: Language,
        spans: &mut Vec<HighlightSpan>,
    ) {
        let node = cursor.node();
        let kind = node.kind();

        // tree-sitter cursors iterate all nodes. Leaf nodes are usually the tokens.
        if node.child_count() == 0 {
            let token_type = Self::classify_node(kind, lang, source, node.start_byte(), node.end_byte());
            if token_type != TokenType::Plain {
                spans.push(HighlightSpan {
                    start_byte: node.start_byte(),
                    end_byte: node.end_byte(),
                    token_type,
                });
            }
        }

        if cursor.goto_first_child() {
            loop {
                Self::walk_node(cursor, source, lang, spans);
                if !cursor.goto_next_sibling() { break; }
            }
            cursor.goto_parent();
        }
    }

    fn classify_node(kind: &str, _lang: Language, source: &[u8], start: usize, end: usize) -> TokenType {
        // Basic heuristic classification based on node kind strings common in tree-sitter grammars
        // A real implementation would use queries (.scm files)
        match kind {
            "line_comment" | "block_comment" | "comment" => TokenType::Comment,
            "string_literal" | "string" | "string_content" | "raw_string_literal"
            | "char_literal" | "template_string" => TokenType::String,
            "integer_literal" | "float_literal" | "number" => TokenType::Number,
            "true" | "false" | "none" | "None" | "null" => TokenType::Constant,
            "type_identifier" | "primitive_type" | "builtin_type" => TokenType::Type,
            "identifier" => {
                let text = std::str::from_utf8(&source[start..end]).unwrap_or("");
                if Self::is_keyword(text) { TokenType::Keyword } else { TokenType::Variable }
            }
            // Keywords often appear as their own node types or anonymous nodes
            "fn" | "let" | "mut" | "pub" | "use" | "mod" | "struct" | "enum" | "impl" | "trait"
            | "const" | "static" | "if" | "else" | "match" | "for" | "while" | "loop" | "return"
            | "async" | "await" | "self" | "super" | "crate" | "where" | "as"
            | "function" | "var" | "class" | "import" | "export" | "from" | "def" | "lambda"
            | "yield" | "try" | "except" | "finally" | "raise" | "with" => TokenType::Keyword,

            "(" | ")" | "[" | "]" | "{" | "}" | ";" | "," | "." | "::" => TokenType::Punctuation,

            "+" | "-" | "*" | "/" | "%" | "=" | "!" | "<" | ">" | "&" | "|" | "^" | "~"
            | "==" | "!=" | "<=" | ">=" | "&&" | "||" | "->" | "=>" => TokenType::Operator,

            "attribute_item" | "decorator" => TokenType::Attribute,
            "macro_invocation" => TokenType::Macro,

            _ => TokenType::Plain,
        }
    }

    fn is_keyword(text: &str) -> bool {
        matches!(text,
            "fn" | "let" | "mut" | "pub" | "use" | "mod" | "struct" | "enum" | "impl"
            | "trait" | "const" | "static" | "if" | "else" | "match" | "for" | "while" | "loop"
            | "return" | "async" | "await" | "self" | "super" | "crate" | "where" | "as" | "in"
            | "ref" | "move" | "type" | "unsafe" | "extern" | "dyn" | "true" | "false"
            | "function" | "var" | "class" | "import" | "export" | "from" | "def" | "yield"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SyntaxParser, Language};

    #[test]
    fn highlight_rust_function() {
        let code = "fn main() { let x = 42; }";
        let mut parser = SyntaxParser::new(Language::Rust).unwrap();
        let tree = parser.parse(code).unwrap();
        let spans = Highlighter::highlight(&tree, code.as_bytes(), Language::Rust);

        // Check for 'fn' keyword
        assert!(spans.iter().any(|s| s.token_type == TokenType::Keyword));
        // Check for '42' number
        assert!(spans.iter().any(|s| s.token_type == TokenType::Number));
        // Check for 'x' variable
        assert!(spans.iter().any(|s| s.token_type == TokenType::Variable));
    }
}
