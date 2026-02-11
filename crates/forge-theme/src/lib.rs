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
pub enum ThemeKind {
    #[default]
    Dark,
    Light,
    HighContrast,
}

impl Theme {
    pub fn color(&self, key: &str) -> Option<[f32; 4]> {
        self.colors.get(key).and_then(|hex| parse_hex_color(hex))
    }
    pub fn default_dark() -> Self {
        builtin::forge_dark()
    }
    pub fn default_light() -> Self {
        builtin::forge_light()
    }
}

pub fn parse_hex_color(hex: &str) -> Option<[f32; 4]> {
    let hex = hex.strip_prefix('#')?;
    let (r, g, b, a) = match hex.len() {
        6 => (
            u8::from_str_radix(&hex[0..2], 16).ok()?,
            u8::from_str_radix(&hex[2..4], 16).ok()?,
            u8::from_str_radix(&hex[4..6], 16).ok()?,
            255u8,
        ),
        8 => (
            u8::from_str_radix(&hex[0..2], 16).ok()?,
            u8::from_str_radix(&hex[2..4], 16).ok()?,
            u8::from_str_radix(&hex[4..6], 16).ok()?,
            u8::from_str_radix(&hex[6..8], 16).ok()?,
        ),
        _ => return None,
    };
    Some([
        r as f32 / 255.0,
        g as f32 / 255.0,
        b as f32 / 255.0,
        a as f32 / 255.0,
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_6_digit() {
        assert_eq!(
            parse_hex_color("#ff8000"),
            Some([1.0, 128.0 / 255.0, 0.0, 1.0])
        );
    }
    #[test]
    fn parse_8_digit() {
        assert!(parse_hex_color("#ff800080").is_some());
    }
    #[test]
    fn default_dark_loads() {
        let t = Theme::default_dark();
        assert!(!t.colors.is_empty());
    }
}
