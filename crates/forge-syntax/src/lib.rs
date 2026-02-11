pub mod colors;
pub mod highlighter;
pub mod language;
pub mod parser;

pub use highlighter::{HighlightSpan, Highlighter, TokenType};
pub use language::Language;
pub use parser::SyntaxParser;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_rust_file() {
        let code = "fn main() { println!(\"Hello\"); }";
        let mut parser = SyntaxParser::new(Language::Rust).unwrap();
        let tree = parser.parse(code).unwrap();
        assert_eq!(tree.root_node().kind(), "source_file");
        assert_eq!(tree.root_node().child(0).unwrap().kind(), "function_item");
    }

    #[test]
    fn parse_json_file() {
        let code = r#"{"name": "forge", "version": 1}"#;
        let mut parser = SyntaxParser::new(Language::Json).unwrap();
        let tree = parser.parse(code).unwrap();
        assert_eq!(tree.root_node().kind(), "document");
    }

    #[test]
    fn language_detection() {
        assert_eq!(Language::from_extension("rs"), Language::Rust);
        assert_eq!(Language::from_extension("js"), Language::JavaScript);
        assert_eq!(Language::from_extension("py"), Language::Python);
        assert_eq!(Language::from_extension("json"), Language::Json);
        assert_eq!(Language::from_extension("unknown"), Language::Unknown);
    }
}
