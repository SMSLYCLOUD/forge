use anyhow::Result;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
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

    /// Forge Day (Light Theme)
    pub fn forge_day() -> Self {
        Self {
            name: "Forge Day".to_string(),
            ui: UiColors {
                editor_bg: Color::new("#f5f5f5").unwrap(),
                sidebar_bg: Color::new("#e5e5e5").unwrap(),
                panel_bg: Color::new("#f5f5f5").unwrap(),
                status_bar_bg: Color::new("#e5e5e5").unwrap(),
                foreground: Color::new("#1a1b26").unwrap(),
                line_number: Color::new("#b0b8d1").unwrap(),
                selection: Color::new("#d5d5d5").unwrap(),
                current_line: Color::new("#eaeaea").unwrap(),
                match_highlight: Color::new("#add8e6").unwrap(),
                border: Color::new("#cccccc").unwrap(),
                cursor: Color::new("#1a1b26").unwrap(),
                active_tab: Color::new("#f5f5f5").unwrap(),
                inactive_tab: Color::new("#e5e5e5").unwrap(),
            },
            syntax: SyntaxColors {
                keyword: Color::new("#7c3aed").unwrap(),
                type_: Color::new("#0891b2").unwrap(),
                function: Color::new("#2563eb").unwrap(),
                string: Color::new("#16a34a").unwrap(),
                number: Color::new("#d97706").unwrap(),
                comment: Color::new("#8389a3").unwrap(),
                constant: Color::new("#b45309").unwrap(),
                macro_: Color::new("#7c3aed").unwrap(),
            },
            diagnostics: DiagnosticColors {
                error: Color::new("#dc2626").unwrap(),
                warning: Color::new("#d97706").unwrap(),
                info: Color::new("#2563eb").unwrap(),
            },
        }
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
