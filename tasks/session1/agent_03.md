# Agent 03 — forge-theme Full Implementation + forge-icons

> **Read `tasks/GLOBAL_RULES.md` first.**

## Task A: Implement forge-theme

Replace stub with full theme engine. Support 100+ named color slots matching VS Code theme format.

### `crates/forge-theme/src/lib.rs`
```rust
//! Theme engine for Forge — loads VS Code-compatible color themes.
mod builtin;
mod token;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use token::TokenColor;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    #[serde(default)]
    pub kind: ThemeKind,
    pub colors: HashMap<String, String>,
    #[serde(default)]
    pub token_colors: Vec<TokenColor>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub enum ThemeKind { #[default] Dark, Light, HighContrast }

impl Theme {
    pub fn color(&self, key: &str) -> Option<[f32; 4]> {
        self.colors.get(key).and_then(|hex| parse_hex_color(hex))
    }
    pub fn default_dark() -> Self { builtin::forge_dark() }
    pub fn default_light() -> Self { builtin::forge_light() }
}

pub fn parse_hex_color(hex: &str) -> Option<[f32; 4]> {
    let hex = hex.strip_prefix('#')?;
    let (r, g, b, a) = match hex.len() {
        6 => (u8::from_str_radix(&hex[0..2], 16).ok()?, u8::from_str_radix(&hex[2..4], 16).ok()?,
              u8::from_str_radix(&hex[4..6], 16).ok()?, 255u8),
        8 => (u8::from_str_radix(&hex[0..2], 16).ok()?, u8::from_str_radix(&hex[2..4], 16).ok()?,
              u8::from_str_radix(&hex[4..6], 16).ok()?, u8::from_str_radix(&hex[6..8], 16).ok()?),
        _ => return None,
    };
    Some([r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a as f32 / 255.0])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn parse_6_digit() { assert_eq!(parse_hex_color("#ff8000"), Some([1.0, 128.0/255.0, 0.0, 1.0])); }
    #[test] fn parse_8_digit() { assert!(parse_hex_color("#ff800080").is_some()); }
    #[test] fn default_dark_loads() { let t = Theme::default_dark(); assert!(!t.colors.is_empty()); }
}
```

### `crates/forge-theme/src/token.rs`
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenColor {
    pub scope: Vec<String>,
    pub settings: TokenSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenSettings {
    pub foreground: Option<String>,
    pub font_style: Option<String>,
}
```

### `crates/forge-theme/src/builtin.rs`
Create `forge_dark()` and `forge_light()` functions returning `Theme` with VS Code Dark+ equivalent colors:
- `editor.background`, `editor.foreground`, `editor.lineHighlightBackground`
- `activityBar.background`, `sideBar.background`, `statusBar.background`
- `tab.activeBackground`, `tab.inactiveBackground`
- Plus 50+ more standard VS Code color keys
- Token colors for: keyword, function, type, string, number, comment, operator, variable

## Task B: Add forge-icons Crate

### `crates/forge-icons/Cargo.toml` + `src/lib.rs`
```rust
//! File type and UI icons.

pub enum FileIcon { Rust, JavaScript, TypeScript, Python, Go, C, Cpp, Json, Toml, Yaml, Html, Css,
    Markdown, Shell, Docker, Git, Generic }

impl FileIcon {
    pub fn from_extension(ext: &str) -> Self {
        match ext {
            "rs" => Self::Rust, "js" | "mjs" | "cjs" => Self::JavaScript,
            "ts" | "tsx" => Self::TypeScript, "py" => Self::Python,
            "go" => Self::Go, "c" | "h" => Self::C, "cpp" | "hpp" | "cc" => Self::Cpp,
            "json" => Self::Json, "toml" => Self::Toml, "yaml" | "yml" => Self::Yaml,
            "html" | "htm" => Self::Html, "css" | "scss" => Self::Css,
            "md" => Self::Markdown, "sh" | "bash" | "zsh" | "ps1" => Self::Shell,
            "dockerfile" => Self::Docker, _ => Self::Generic,
        }
    }
    pub fn glyph(&self) -> &'static str {
        match self {
            Self::Rust => "🦀", Self::JavaScript => "📜", Self::TypeScript => "🔷",
            Self::Python => "🐍", Self::Go => "🔵", Self::C | Self::Cpp => "⚙️",
            Self::Json => "📋", Self::Toml => "⚙️", Self::Yaml => "📄",
            Self::Html => "🌐", Self::Css => "🎨", Self::Markdown => "📝",
            Self::Shell => "💻", Self::Docker => "🐳", Self::Git => "📦",
            Self::Generic => "📄",
        }
    }
}

pub enum UiIcon { Folder, FolderOpen, Search, Settings, Git, Debug, Extensions, Terminal }

impl UiIcon {
    pub fn glyph(&self) -> &'static str {
        match self {
            Self::Folder => "📁", Self::FolderOpen => "📂", Self::Search => "🔍",
            Self::Settings => "⚙️", Self::Git => "📦", Self::Debug => "🐛",
            Self::Extensions => "🧩", Self::Terminal => "💻",
        }
    }
}
```

Add both crates to workspace members. Tests for all icon mappings.

**Acceptance**: `cargo test -p forge-theme -p forge-icons` passes.
