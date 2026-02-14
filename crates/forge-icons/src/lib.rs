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
        // Mapped to Codicon generic file icons
        match self {
            Self::Rust | Self::JavaScript | Self::TypeScript | Self::React | Self::Vue |
            Self::Python | Self::Go | Self::C | Self::Cpp | Self::Java | Self::Kotlin |
            Self::Swift | Self::Ruby | Self::PHP | Self::Json | Self::Toml | Self::Yaml |
            Self::Xml | Self::Html | Self::Css | Self::Sass | Self::Markdown | Self::Shell |
            Self::Bat | Self::Docker | Self::Git | Self::License | Self::Readme => "\u{ea77}", // file-code

            Self::Image | Self::Audio | Self::Video => "\u{ea78}", // file-media
            Self::Archive => "\u{ea7d}", // file-zip
            Self::Database | Self::Lock => "\u{ea76}", // file-binary
            Self::Generic => "\u{ea7b}", // file (generic)
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
    Account,
    Terminal,
    Error,
    Warning,
    Info,
    Check,
    Close,
    Add,
    Edit,
    Delete,
    ChevronRight,
    ChevronDown,
}

impl UiIcon {
    /// Returns the Codicon unicode character
    pub fn glyph(&self) -> &'static str {
        match self {
            // VS Code Codicons
            // https://microsoft.github.io/vscode-codicons/dist/codicon.html
            Self::Folder => "\u{ea83}",         // folder
            Self::FolderOpen => "\u{ea84}",     // folder-opened
            Self::Search => "\u{ea6d}",         // search
            Self::SourceControl => "\u{ea68}",  // source-control
            Self::Debug => "\u{ea71}",          // debug-alt
            Self::Extensions => "\u{ea6b}",     // extensions
            Self::AiAgent => "\u{ea44}",        // robot (closest)
            Self::Settings => "\u{ea6e}",       // settings-gear
            Self::Account => "\u{ea60}",        // account
            Self::Terminal => "\u{ea85}",       // terminal
            Self::Error => "\u{ea87}",          // error
            Self::Warning => "\u{ea6c}",        // warning
            Self::Info => "\u{ea74}",           // info
            Self::Check => "\u{ea5e}",          // check
            Self::Close => "\u{ea76}",          // close
            Self::Add => "\u{ea60}",            // add
            Self::Edit => "\u{ea73}",           // edit
            Self::Delete => "\u{ea81}",         // trash
            Self::ChevronRight => "\u{ea61}",   // chevron-right
            Self::ChevronDown => "\u{ea5e}",    // chevron-down
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
        assert_eq!(FileIcon::Rust.glyph(), "\u{ea77}");
        assert_eq!(UiIcon::Folder.glyph(), "\u{ea83}");
        assert_eq!(UiIcon::SourceControl.glyph(), "\u{ea68}");
    }

    #[test]
    fn test_colors() {
        let c = FileIcon::Rust.color();
        assert!(c[0] > 0.5); // somewhat red
    }
}
