//! File type and UI icons.

pub enum FileIcon {
    Rust,
    JavaScript,
    TypeScript,
    Python,
    Go,
    C,
    Cpp,
    Json,
    Toml,
    Yaml,
    Html,
    Css,
    Markdown,
    Shell,
    Docker,
    Git,
    Generic,
}

impl FileIcon {
    pub fn from_extension(ext: &str) -> Self {
        match ext {
            "rs" => Self::Rust,
            "js" | "mjs" | "cjs" => Self::JavaScript,
            "ts" | "tsx" => Self::TypeScript,
            "py" => Self::Python,
            "go" => Self::Go,
            "c" | "h" => Self::C,
            "cpp" | "hpp" | "cc" => Self::Cpp,
            "json" => Self::Json,
            "toml" => Self::Toml,
            "yaml" | "yml" => Self::Yaml,
            "html" | "htm" => Self::Html,
            "css" | "scss" => Self::Css,
            "md" => Self::Markdown,
            "sh" | "bash" | "zsh" | "ps1" => Self::Shell,
            "dockerfile" => Self::Docker,
            _ => Self::Generic,
        }
    }
    pub fn glyph(&self) -> &'static str {
        match self {
            Self::Rust => "🦀",
            Self::JavaScript => "📜",
            Self::TypeScript => "🔷",
            Self::Python => "🐍",
            Self::Go => "🔵",
            Self::C | Self::Cpp => "⚙️",
            Self::Json => "📋",
            Self::Toml => "⚙️",
            Self::Yaml => "📄",
            Self::Html => "🌐",
            Self::Css => "🎨",
            Self::Markdown => "📝",
            Self::Shell => "💻",
            Self::Docker => "🐳",
            Self::Git => "📦",
            Self::Generic => "📄",
        }
    }
}

pub enum UiIcon {
    Folder,
    FolderOpen,
    Search,
    Settings,
    Git,
    Debug,
    Extensions,
    Terminal,
}

impl UiIcon {
    pub fn glyph(&self) -> &'static str {
        match self {
            Self::Folder => "📁",
            Self::FolderOpen => "📂",
            Self::Search => "🔍",
            Self::Settings => "⚙️",
            Self::Git => "📦",
            Self::Debug => "🐛",
            Self::Extensions => "🧩",
            Self::Terminal => "💻",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_extension_mapping() {
        assert!(matches!(FileIcon::from_extension("rs"), FileIcon::Rust));
        assert!(matches!(FileIcon::from_extension("cpp"), FileIcon::Cpp));
    }

    #[test]
    fn test_glyphs() {
        assert_eq!(FileIcon::Rust.glyph(), "🦀");
        assert_eq!(UiIcon::Folder.glyph(), "📁");
    }
}
