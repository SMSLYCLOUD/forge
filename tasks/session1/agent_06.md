# Agent 06 — forge-syntax Highlighter + Syntax Rendering

> **Read `tasks/GLOBAL_RULES.md` first.**

## Task A: Tree-sitter Highlight Query System

### `crates/forge-syntax/src/highlighter.rs`
```rust
use crate::language::Language;
use anyhow::Result;

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
        match kind {
            "line_comment" | "block_comment" | "comment" => TokenType::Comment,
            "string_literal" | "string" | "string_content" | "raw_string_literal"
            | "char_literal" | "template_string" => TokenType::String,
            "integer_literal" | "float_literal" | "number" => TokenType::Number,
            "true" | "false" | "none" | "None" | "null" => TokenType::Constant,
            "type_identifier" | "primitive_type" | "builtin_type" => TokenType::Type,
            "identifier" => {
                let text = std::str::from_utf8(&source[start..end]).unwrap_or("");
                if is_keyword(text) { TokenType::Keyword } else { TokenType::Variable }
            }
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
}

fn is_keyword(text: &str) -> bool {
    matches!(text, "fn" | "let" | "mut" | "pub" | "use" | "mod" | "struct" | "enum" | "impl"
        | "trait" | "const" | "static" | "if" | "else" | "match" | "for" | "while" | "loop"
        | "return" | "async" | "await" | "self" | "super" | "crate" | "where" | "as" | "in"
        | "ref" | "move" | "type" | "unsafe" | "extern" | "dyn" | "true" | "false"
        | "function" | "var" | "class" | "import" | "export" | "from" | "def" | "yield")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SyntaxParser;
    #[test]
    fn highlight_rust_function() {
        let code = "fn main() { let x = 42; }";
        let mut parser = SyntaxParser::new(Language::Rust).unwrap();
        let tree = parser.parse(code).unwrap();
        let spans = Highlighter::highlight(&tree, code.as_bytes(), Language::Rust);
        assert!(spans.iter().any(|s| s.token_type == TokenType::Keyword));
        assert!(spans.iter().any(|s| s.token_type == TokenType::Number));
    }
}
```

Add `pub mod highlighter;` and `pub use highlighter::{Highlighter, HighlightSpan, TokenType};` to `lib.rs`.

## Task B: Token Color Mapping

### `crates/forge-syntax/src/colors.rs`
Map `TokenType` to default colors (Dracula-style):
```rust
pub fn default_color(token: TokenType) -> [u8; 3] {
    match token {
        TokenType::Keyword => [255, 121, 198],   // pink
        TokenType::Function => [80, 250, 123],   // green
        TokenType::Type => [139, 233, 253],      // cyan
        TokenType::String => [241, 250, 140],    // yellow
        TokenType::Number => [189, 147, 249],    // purple
        TokenType::Comment => [98, 114, 164],    // gray
        TokenType::Operator => [255, 184, 108],  // orange
        TokenType::Variable => [248, 248, 242],  // white
        TokenType::Constant => [189, 147, 249],  // purple
        TokenType::Punctuation => [248, 248, 242],
        _ => [248, 248, 242],
    }
}
```

**Acceptance**: `cargo test -p forge-syntax` passes with all highlighter tests.
