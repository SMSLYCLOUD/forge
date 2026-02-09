# Agent 05 — forge-syntax: Tree-sitter Parser + Language Detection

> **Read `tasks/GLOBAL_RULES.md` first.**

## Task A: Tree-sitter Parser Wrapper

Add `tree-sitter = "0.24"` and `tree-sitter-rust = "0.23"`, `tree-sitter-javascript = "0.23"`, `tree-sitter-python = "0.23"`, `tree-sitter-json = "0.24"`, `tree-sitter-toml = "0.24"` to workspace dependencies.

### Create `crates/forge-syntax/Cargo.toml`
```toml
[package]
name = "forge-syntax"
version.workspace = true
edition.workspace = true
[dependencies]
tree-sitter = { workspace = true }
tree-sitter-rust = { workspace = true }
tree-sitter-javascript = { workspace = true }
tree-sitter-python = { workspace = true }
tree-sitter-json = { workspace = true }
tree-sitter-toml = { workspace = true }
anyhow = { workspace = true }
thiserror = { workspace = true }
```

### `crates/forge-syntax/src/language.rs`
```rust
use anyhow::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    Rust, JavaScript, TypeScript, Python, Go, C, Cpp,
    Json, Toml, Yaml, Html, Css, Markdown, Shell, Unknown,
}

impl Language {
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "rs" => Self::Rust, "js" | "mjs" | "cjs" | "jsx" => Self::JavaScript,
            "ts" | "tsx" => Self::TypeScript, "py" | "pyw" => Self::Python,
            "go" => Self::Go, "c" | "h" => Self::C, "cpp" | "hpp" | "cc" | "cxx" => Self::Cpp,
            "json" => Self::Json, "toml" => Self::Toml, "yaml" | "yml" => Self::Yaml,
            "html" | "htm" => Self::Html, "css" | "scss" => Self::Css,
            "md" | "markdown" => Self::Markdown, "sh" | "bash" | "zsh" => Self::Shell,
            _ => Self::Unknown,
        }
    }

    pub fn from_path(path: &str) -> Self {
        std::path::Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .map(Self::from_extension)
            .unwrap_or(Self::Unknown)
    }

    pub fn tree_sitter_language(&self) -> Option<tree_sitter::Language> {
        match self {
            Self::Rust => Some(tree_sitter_rust::LANGUAGE.into()),
            Self::JavaScript | Self::TypeScript => Some(tree_sitter_javascript::LANGUAGE.into()),
            Self::Python => Some(tree_sitter_python::LANGUAGE.into()),
            Self::Json => Some(tree_sitter_json::LANGUAGE.into()),
            Self::Toml => Some(tree_sitter_toml::LANGUAGE.into()),
            _ => None,
        }
    }
}
```

### `crates/forge-syntax/src/parser.rs`
```rust
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
        self.parser.parse(text, None)
            .ok_or_else(|| anyhow::anyhow!("Parse failed for {:?}", self.language))
    }

    pub fn reparse(&mut self, text: &str, old_tree: &tree_sitter::Tree) -> Result<tree_sitter::Tree> {
        self.parser.parse(text, Some(old_tree))
            .ok_or_else(|| anyhow::anyhow!("Reparse failed"))
    }

    pub fn language(&self) -> Language { self.language }
}
```

### `crates/forge-syntax/src/lib.rs`
```rust
pub mod language;
pub mod parser;
pub use language::Language;
pub use parser::SyntaxParser;
```

**Tests**: Parse a simple Rust file, verify tree is not null. Parse JSON, verify valid tree. Detect language from extensions (.rs, .js, .py).

**Acceptance**: `cargo test -p forge-syntax` passes.
