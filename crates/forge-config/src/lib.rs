use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub editor: EditorConfig,
    #[serde(default)]
    pub typography: TypographyConfig,
    #[serde(default)]
    pub terminal: TerminalConfig,
    #[serde(default)]
    pub keybindings: KeybindingConfig,
    #[serde(default)]
    pub onboarding: OnboardingConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            editor: EditorConfig::default(),
            typography: TypographyConfig::default(),
            terminal: TerminalConfig::default(),
            keybindings: KeybindingConfig::default(),
            onboarding: OnboardingConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorConfig {
    pub theme: String,
    pub auto_save: bool,
    pub tab_size: usize,
    pub word_wrap: bool,
    pub format_on_save: bool,
    pub minimap: bool,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            theme: "forge-night".to_string(),
            auto_save: true,
            tab_size: 4,
            word_wrap: false,
            format_on_save: true,
            minimap: false, // Per research: most devs don't use it
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypographyConfig {
    pub font_family: String,
    pub font_size: f32,
    pub line_height: f32,
    pub letter_spacing: f32,
    pub ligatures: bool,
}

impl Default for TypographyConfig {
    fn default() -> Self {
        Self {
            font_family: "JetBrains Mono".to_string(),
            font_size: 14.0,
            line_height: 1.55,
            letter_spacing: 0.5,
            ligatures: false, // Per spec: OFF by default
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalConfig {
    pub shell: Option<String>,
    pub font_family: Option<String>,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            shell: None,       // Auto-detect
            font_family: None, // Same as editor
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindingConfig {
    pub mode: String, // "standard" or "modal"
}

impl Default for KeybindingConfig {
    fn default() -> Self {
        Self {
            mode: "standard".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingConfig {
    pub hints: bool,
}

impl Default for OnboardingConfig {
    fn default() -> Self {
        Self { hints: true }
    }
}

impl Config {
    pub fn load_from_str(s: &str) -> Result<Self> {
        Ok(toml::from_str(s)?)
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let s = std::fs::read_to_string(path)?;
        Self::load_from_str(&s)
    }
}
