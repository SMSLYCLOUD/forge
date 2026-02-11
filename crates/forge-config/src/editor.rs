use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct EditorConfig {
    pub tab_size: usize,
    pub insert_spaces: bool,
    pub word_wrap: bool,
    pub line_numbers: bool,
    pub minimap: bool,
    pub auto_save_delay_ms: u64,
    pub cursor_blink: bool,
    pub bracket_matching: bool,
    pub indent_guides: bool,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            tab_size: 4,
            insert_spaces: true,
            word_wrap: false,
            line_numbers: true,
            minimap: true,
            auto_save_delay_ms: 30000,
            cursor_blink: true,
            bracket_matching: true,
            indent_guides: true,
        }
    }
}
