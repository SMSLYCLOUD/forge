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
            // Self::Toml => Some(tree_sitter_toml::language()), // Disable TOML for now due to version mismatch
            _ => None,
        }
    }
}
