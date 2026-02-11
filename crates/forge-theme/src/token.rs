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
