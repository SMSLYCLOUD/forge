use crate::language::Language;
use anyhow::Result;

pub struct SyntaxParser {
    parser: tree_sitter::Parser,
    language: Language,
}

impl SyntaxParser {
    pub fn new(language: Language) -> Result<Self> {
        let mut parser = tree_sitter::Parser::new();
        if let Some(ts_lang) = language.tree_sitter_language() {
            parser.set_language(&ts_lang)?;
        }
        Ok(Self { parser, language })
    }

    pub fn parse(&mut self, text: &str) -> Result<tree_sitter::Tree> {
        self.parser
            .parse(text, None)
            .ok_or_else(|| anyhow::anyhow!("Parse failed for {:?}", self.language))
    }

    pub fn reparse(
        &mut self,
        text: &str,
        old_tree: &tree_sitter::Tree,
    ) -> Result<tree_sitter::Tree> {
        self.parser
            .parse(text, Some(old_tree))
            .ok_or_else(|| anyhow::anyhow!("Reparse failed"))
    }

    pub fn language(&self) -> Language {
        self.language
    }
}
