//! File type and UI icons with VS Code/Material Icon Theme inspiration.

pub enum FileIcon {
    Rust,
    JavaScript,
    TypeScript,
    React,
    Vue,
    Python,
    Go,
    C,
    Cpp,
    Java,
    Kotlin,
    Swift,
    Ruby,
    PHP,
    Json,
    Toml,
    Yaml,
    Xml,
    Html,
    Css,
    Sass,
    Markdown,
    Shell,
    Bat,
    Docker,
    Git,
    Image,
    Audio,
    Video,
    Archive,
    Database,
    Lock,
    License,
    Readme,
    Generic,
}

impl FileIcon {
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "rs" => Self::Rust,
            "js" | "mjs" | "cjs" => Self::JavaScript,
            "ts" | "tsx" => Self::TypeScript, // React often uses .tsx
            "jsx" => Self::React,
            "vue" => Self::Vue,
            "py" | "pyw" => Self::Python,
            "go" => Self::Go,
            "c" | "h" => Self::C,
            "cpp" | "hpp" | "cc" | "cxx" => Self::Cpp,
            "java" | "jar" => Self::Java,
            "kt" | "kts" => Self::Kotlin,
            "swift" => Self::Swift,
            "rb" | "erb" => Self::Ruby,
            "php" => Self::PHP,
            "json" | "jsonc" => Self::Json,
            "toml" => Self::Toml,
            "yaml" | "yml" => Self::Yaml,
            "xml" | "xaml" | "svg" => Self::Xml,
            "html" | "htm" => Self::Html,
            "css" => Self::Css,
            "scss" | "sass" | "less" => Self::Sass,
            "md" | "markdown" => Self::Markdown,
            "sh" | "bash" | "zsh" | "fish" => Self::Shell,
            "bat" | "cmd" | "ps1" => Self::Bat,
            "dockerfile" | "containerfile" => Self::Docker,
            "git" | "gitignore" | "gitattributes" => Self::Git,
            "png" | "jpg" | "jpeg" | "gif" | "bmp" | "ico" | "webp" => Self::Image,
            "mp3" | "wav" | "ogg" | "flac" => Self::Audio,
            "mp4" | "mov" | "avi" | "mkv" | "webm" => Self::Video,
            "zip" | "tar" | "gz" | "7z" | "rar" => Self::Archive,
            "db" | "sqlite" | "sql" => Self::Database,
            "lock" => Self::Lock,
            "license" => Self::License,
            _ => Self::Generic,
        }
    }

    pub fn from_filename(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "cargo.toml" | "cargo.lock" => Self::Rust,
            "package.json" | "package-lock.json" => Self::JavaScript,
            "tsconfig.json" => Self::TypeScript,
            "dockerfile" => Self::Docker,
            "makefile" => Self::Shell,
            "license" | "license.md" | "license.txt" => Self::License,
            "readme.md" | "readme.txt" => Self::Readme,
            _ => {
                if let Some(ext) = std::path::Path::new(name).extension() {
                    Self::from_extension(&ext.to_string_lossy())
                } else {
                    Self::Generic
                }
            }
        }
    }

    pub fn glyph(&self) -> &'static str {
        match self {
            Self::Rust => "🦀",
            Self::JavaScript => "📜", // JS logo is hard, scroll is classic script
            Self::TypeScript => "🔷", // TS blue
            Self::React => "⚛️",
            Self::Vue => "🇻",
            Self::Python => "🐍",
            Self::Go => "🐹", // No gopher, hamster is close? Or just blue dot
            Self::C => "🇨",
            Self::Cpp => "🇨", // C++
            Self::Java => "☕",
            Self::Kotlin => "🇰",
            Self::Swift => "🐦", // Swift bird
            Self::Ruby => "💎",
            Self::PHP => "🐘",
            Self::Json => "📋",
            Self::Toml => "⚙️",
            Self::Yaml => "📄",
            Self::Xml => "📰",
            Self::Html => "🌐",
            Self::Css => "🎨",
            Self::Sass => "💅",
            Self::Markdown => "📝",
            Self::Shell => "💻",
            Self::Bat => "⌨️",
            Self::Docker => "🐳",
            Self::Git => "🌲", // Git graph
            Self::Image => "🖼️",
            Self::Audio => "🎵",
            Self::Video => "🎬",
            Self::Archive => "📦",
            Self::Database => "🗄️",
            Self::Lock => "🔒",
            Self::License => "⚖️",
            Self::Readme => "📖",
            Self::Generic => "📄",
        }
    }

    /// Color hex code (approximate for Material Icon Theme)
    pub fn color(&self) -> [f32; 4] {
        match self {
            Self::Rust => [0.8, 0.3, 0.1, 1.0],              // Orange
            Self::JavaScript => [0.95, 0.9, 0.2, 1.0],       // Yellow
            Self::TypeScript => [0.0, 0.48, 0.8, 1.0],       // Blue
            Self::React => [0.38, 0.85, 1.0, 1.0],           // Cyan
            Self::Vue => [0.25, 0.72, 0.49, 1.0],            // Green
            Self::Python => [0.2, 0.4, 0.6, 1.0],            // Blue-ish
            Self::Go => [0.0, 0.68, 0.84, 1.0],              // Cyan
            Self::C | Self::Cpp => [0.0, 0.36, 0.68, 1.0],   // Blue
            Self::Java => [0.94, 0.1, 0.1, 1.0],             // Red
            Self::Kotlin => [0.5, 0.2, 0.8, 1.0],            // Purple
            Self::Swift => [0.98, 0.18, 0.11, 1.0],          // Orange-Red
            Self::Ruby => [0.8, 0.1, 0.1, 1.0],              // Red
            Self::PHP => [0.46, 0.5, 0.68, 1.0],             // Purple
            Self::Json => [0.96, 0.8, 0.1, 1.0],             // Yellow
            Self::Toml => [0.6, 0.6, 0.6, 1.0],              // Grey
            Self::Yaml => [0.6, 0.6, 0.6, 1.0],              // Grey
            Self::Xml | Self::Html => [0.89, 0.3, 0.1, 1.0], // Orange
            Self::Css | Self::Sass => [0.1, 0.6, 0.9, 1.0],  // Blue
            Self::Markdown | Self::Readme | Self::License => [0.5, 0.5, 0.5, 1.0], // Grey
            Self::Shell | Self::Bat => [0.3, 0.3, 0.3, 1.0], // Dark Grey
            Self::Docker => [0.0, 0.5, 1.0, 1.0],            // Blue
            Self::Git => [0.9, 0.3, 0.2, 1.0],               // Red
            Self::Image | Self::Audio | Self::Video => [0.6, 0.4, 0.8, 1.0], // Purple
            Self::Archive | Self::Database | Self::Lock => [0.5, 0.5, 0.5, 1.0], // Grey
            Self::Generic => [0.8, 0.8, 0.8, 1.0],           // Light Grey
        }
    }
}

pub enum UiIcon {
    Folder,
    FolderOpen,
    Search,
    SourceControl,
    Debug,
    Extensions,
    AiAgent,
    Settings,
    Terminal,
    Error,
    Warning,
    Info,
    Check,
    Close,
    Add,
    Edit,
    Delete,
}

impl UiIcon {
    pub fn glyph(&self) -> &'static str {
        match self {
            Self::Folder => "📁",
            Self::FolderOpen => "📂",
            Self::Search => "🔍",
            Self::SourceControl => "⎇",
            Self::Debug => "🐞",
            Self::Extensions => "🧩",
            Self::AiAgent => "🤖",
            Self::Settings => "⚙",
            Self::Terminal => "💻",
            Self::Error => "✕",
            Self::Warning => "⚠",
            Self::Info => "ℹ",
            Self::Check => "✓",
            Self::Close => "✕",
            Self::Add => "＋",
            Self::Edit => "✎",
            Self::Delete => "🗑️",
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
        assert!(matches!(
            FileIcon::from_filename("Cargo.toml"),
            FileIcon::Rust
        ));
    }

    #[test]
    fn test_glyphs() {
        assert_eq!(FileIcon::Rust.glyph(), "🦀");
        assert_eq!(UiIcon::Folder.glyph(), "📁");
        assert_eq!(UiIcon::SourceControl.glyph(), "⎇");
    }

    #[test]
    fn test_colors() {
        let c = FileIcon::Rust.color();
        assert!(c[0] > 0.5); // somewhat red
    }
}
