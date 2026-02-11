//! Forge configuration — TOML-based with sensible defaults.
mod editor;
mod terminal;
pub use editor::EditorConfig;
pub use terminal::TerminalConfig;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ForgeConfig {
    pub editor: EditorConfig,
    pub terminal: TerminalConfig,
    pub theme: String,
    pub font_family: String,
    pub font_size: f32,
}

impl Default for ForgeConfig {
    fn default() -> Self {
        Self {
            editor: EditorConfig::default(),
            terminal: TerminalConfig::default(),
            theme: "Forge Dark".into(),
            font_family: "Cascadia Code".into(),
            font_size: 14.0,
        }
    }
}

impl ForgeConfig {
    pub fn load() -> Result<Self> {
        Self::load_from(&Self::config_path())
    }

    pub fn load_from(path: &std::path::Path) -> Result<Self> {
        if path.exists() {
            let text = std::fs::read_to_string(path)?;
            Ok(toml::from_str(&text)?)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        Self::save_to(self, &Self::config_path())
    }

    pub fn save_to(&self, path: &std::path::Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, toml::to_string_pretty(self)?)?;
        Ok(())
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
    pub fn config_path() -> PathBuf {
        dirs_next::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("forge")
            .join("config.toml")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_default_config() {
        let config = ForgeConfig::default();
        assert_eq!(config.editor.tab_size, 4);
        assert_eq!(config.terminal.font_size, 13.0);
    }

    #[test]
    fn test_save_load_roundtrip() -> Result<()> {
        let temp_dir = std::env::temp_dir().join("forge_test_config");
        let config_path = temp_dir.join("config.toml");

impl Default for OnboardingConfig {
    fn default() -> Self {
        Self { hints: true }
    }
}
        // Clean up previous run if any
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir)?;
        }
        fs::create_dir_all(&temp_dir)?;

        let mut config = ForgeConfig::default();
        config.theme = "Custom Theme".into();
        config.editor.tab_size = 2;

        config.save_to(&config_path)?;

        let loaded_config = ForgeConfig::load_from(&config_path)?;

        assert_eq!(loaded_config.theme, "Custom Theme");
        assert_eq!(loaded_config.editor.tab_size, 2);

        fs::remove_dir_all(temp_dir)?;
        Ok(())
    }
}
