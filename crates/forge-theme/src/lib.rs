use anyhow::Result;
use serde::{Deserialize, Serialize, Serializer, Deserializer};

/// Wrapper around hex string color to support Serde
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Color(String);

impl Color {
    pub fn new(hex: &str) -> Result<Self> {
        // Basic validation
        if hex.starts_with('#') && (hex.len() == 4 || hex.len() == 7) {
            Ok(Self(hex.to_string()))
        } else {
            Err(anyhow::anyhow!("Invalid hex color: {}", hex))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Serialize for Color {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Color::new(&s).map_err(serde::de::Error::custom)
    }
}

/// Semantic syntax highlighting colors (the 8-color rule)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntaxColors {
    pub keyword: Color,
    pub type_: Color,
    pub function: Color,
    pub string: Color,
    pub number: Color,
    pub comment: Color,
    pub constant: Color,
    pub macro_: Color,
}

/// UI component colors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiColors {
    pub editor_bg: Color,
    pub sidebar_bg: Color,
    pub panel_bg: Color,
    pub status_bar_bg: Color,
    pub foreground: Color,
    pub line_number: Color,
    pub selection: Color,
    pub current_line: Color,
    pub match_highlight: Color,
    pub border: Color,
    pub cursor: Color,
    pub active_tab: Color,
    pub inactive_tab: Color,
}

/// Diagnostic colors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticColors {
    pub error: Color,
    pub warning: Color,
    pub info: Color,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub ui: UiColors,
    pub syntax: SyntaxColors,
    pub diagnostics: DiagnosticColors,
}

impl Theme {
    /// Forge Night (Dark Theme) - CIELAB calibrated
    pub fn forge_night() -> Self {
        Self {
            name: "Forge Night".to_string(),
            ui: UiColors {
                editor_bg: Color::new("#1a1b26").unwrap(),
                sidebar_bg: Color::new("#16161e").unwrap(),
                panel_bg: Color::new("#1a1b26").unwrap(),
                status_bar_bg: Color::new("#16161e").unwrap(),
                foreground: Color::new("#c0caf5").unwrap(),
                line_number: Color::new("#3b4261").unwrap(),
                selection: Color::new("#283457").unwrap(),
                current_line: Color::new("#1e2030").unwrap(),
                match_highlight: Color::new("#3d59a1").unwrap(),
                border: Color::new("#27293d").unwrap(),
                cursor: Color::new("#c0caf5").unwrap(),
                active_tab: Color::new("#1a1b26").unwrap(),
                inactive_tab: Color::new("#16161e").unwrap(),
            },
            syntax: SyntaxColors {
                keyword: Color::new("#9d7cd8").unwrap(),
                type_: Color::new("#2ac3de").unwrap(),
                function: Color::new("#7aa2f7").unwrap(),
                string: Color::new("#9ece6a").unwrap(),
                number: Color::new("#ff9e64").unwrap(),
                comment: Color::new("#565f89").unwrap(),
                constant: Color::new("#e0af68").unwrap(),
                macro_: Color::new("#bb9af7").unwrap(),
            },
            diagnostics: DiagnosticColors {
                error: Color::new("#f7768e").unwrap(),
                warning: Color::new("#e0af68").unwrap(),
                info: Color::new("#7aa2f7").unwrap(),
            },
        }
    }

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
    }
}
