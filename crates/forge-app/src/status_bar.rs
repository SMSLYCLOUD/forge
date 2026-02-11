use crate::ui::{colors, LayoutConstants, Zone};

/// A single item displayed in the status bar
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct StatusItem {
    pub text: String,
    pub tooltip: String,
    pub color: Option<[f32; 4]>,
    pub alignment: StatusAlignment,
    pub priority: i32, // Higher = more important, gets rendered first
    pub click_action: Option<StatusAction>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(dead_code)]
pub enum StatusAlignment {
    Left,
    Right,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum StatusAction {
    ToggleAiPanel,
    ToggleSidebar,
    CycleMode,
    OpenCommandPalette,
    ShowNotifications,
    SelectLanguage,
    SelectEncoding,
    SelectLineEnding,
}

/// Status bar state and rendering
#[allow(dead_code)]
pub struct StatusBar {
    pub items: Vec<StatusItem>,
    /// Current cursor line (1-indexed for display)
    pub cursor_line: usize,
    /// Current cursor column (1-indexed for display)
    pub cursor_col: usize,
    /// File encoding
    pub encoding: String,
    /// File language
    pub language: String,
    /// Line ending style
    pub line_ending: String,
    /// Git branch name
    pub git_branch: Option<String>,
    /// Frame time in ms
    pub frame_time_ms: f32,
    /// Confidence score (0-100)
    pub confidence_score: Option<f32>,
    /// AI agent status
    pub ai_status: String,
    /// Network status
    pub network_status: String,
    /// Current UI mode
    pub mode_indicator: String,
    /// Error count
    pub error_count: usize,
    /// Warning count
    pub warning_count: usize,
}

impl StatusBar {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            cursor_line: 1,
            cursor_col: 1,
            encoding: String::from("UTF-8"),
            language: String::from("Plain Text"),
            line_ending: String::from("LF"),
            git_branch: None,
            frame_time_ms: 0.0,
            confidence_score: None,
            ai_status: String::from("Ready"),
            network_status: String::from("🌐 Online"),
            mode_indicator: String::from("🖥️ Standard"),
            error_count: 0,
            warning_count: 0,
        }
    }

    /// Build the ordered list of status items
    #[allow(dead_code)]
    pub fn build_items(&self) -> Vec<StatusItem> {
        let mut items = Vec::with_capacity(16);

        // LEFT SIDE items

        // Git branch
        if let Some(ref branch) = self.git_branch {
            items.push(StatusItem {
                text: format!("⎇ {}", branch),
                tooltip: format!("Git Branch: {}", branch),
                color: None,
                alignment: StatusAlignment::Left,
                priority: 100,
                click_action: None,
            });
        }

        // Errors and warnings
        if self.error_count > 0 || self.warning_count > 0 {
            items.push(StatusItem {
                text: format!("✕ {}  ⚠ {}", self.error_count, self.warning_count),
                tooltip: format!("{} errors, {} warnings", self.error_count, self.warning_count),
                color: if self.error_count > 0 { Some(colors::ERROR) } else { None },
                alignment: StatusAlignment::Left,
                priority: 90,
                click_action: Some(StatusAction::ShowNotifications),
            });
        }

        // Mode indicator
        items.push(StatusItem {
            text: self.mode_indicator.clone(),
            tooltip: String::from("Click to change UI mode"),
            color: None,
            alignment: StatusAlignment::Left,
            priority: 80,
            click_action: Some(StatusAction::CycleMode),
        });

        // Network status
        items.push(StatusItem {
            text: self.network_status.clone(),
            tooltip: String::from("Network connection status"),
            color: None,
            alignment: StatusAlignment::Left,
            priority: 70,
            click_action: None,
        });

        // RIGHT SIDE items

        // Cursor position
        items.push(StatusItem {
            text: format!("Ln {}, Col {}", self.cursor_line, self.cursor_col),
            tooltip: String::from("Go to Line"),
            color: None,
            alignment: StatusAlignment::Right,
            priority: 100,
            click_action: None,
        });

        // Encoding
        items.push(StatusItem {
            text: self.encoding.clone(),
            tooltip: String::from("Select Encoding"),
            color: None,
            alignment: StatusAlignment::Right,
            priority: 70,
            click_action: Some(StatusAction::SelectEncoding),
        });

        // Line ending
        items.push(StatusItem {
            text: self.line_ending.clone(),
            tooltip: String::from("Select End of Line Sequence"),
            color: None,
            alignment: StatusAlignment::Right,
            priority: 60,
            click_action: Some(StatusAction::SelectLineEnding),
        });

        // Language
        items.push(StatusItem {
            text: self.language.clone(),
            tooltip: String::from("Select Language Mode"),
            color: None,
            alignment: StatusAlignment::Right,
            priority: 50,
            click_action: Some(StatusAction::SelectLanguage),
        });

        // Confidence score
        if let Some(score) = self.confidence_score {
            items.push(StatusItem {
                text: format!("⚡ {:.1}%", score),
                tooltip: format!("Confidence Score: {:.1}%", score),
                color: Some(if score > 80.0 {
                    colors::SUCCESS
                } else if score > 50.0 {
                    colors::WARNING
                } else {
                    colors::ERROR
                }),
                alignment: StatusAlignment::Right,
                priority: 40,
                click_action: None,
            });
        }

        // AI status
        items.push(StatusItem {
            text: format!("🤖 {}", self.ai_status),
            tooltip: String::from("AI Agent Status — click to toggle"),
            color: None,
            alignment: StatusAlignment::Right,
            priority: 30,
            click_action: Some(StatusAction::ToggleAiPanel),
        });

        // Frame time
        items.push(StatusItem {
            text: format!("{:.1}ms", self.frame_time_ms),
            tooltip: String::from("Frame render time"),
            color: Some(if self.frame_time_ms < 7.0 {
                colors::SUCCESS
            } else if self.frame_time_ms < 16.0 {
                colors::WARNING
            } else {
                colors::ERROR
            }),
            alignment: StatusAlignment::Right,
            priority: 10,
            click_action: None,
        });

        items
    }

    /// Get text positions for rendering
    /// Returns (text, x, y, color) tuples
    #[allow(dead_code)]
    pub fn text_positions(&self, zone: &Zone) -> Vec<(String, f32, f32, [f32; 4])> {
        let items = self.build_items();
        let mut result = Vec::with_capacity(items.len());
        let text_y = zone.y + (zone.height - LayoutConstants::SMALL_FONT_SIZE) / 2.0;
        let char_width = LayoutConstants::CHAR_WIDTH;
        let padding = 12.0;

        // Left items
        let mut left_x = zone.x + padding;
        let mut left_items: Vec<&StatusItem> = items.iter()
            .filter(|i| i.alignment == StatusAlignment::Left)
            .collect();
        left_items.sort_by(|a, b| b.priority.cmp(&a.priority));

        for item in &left_items {
            let color = item.color.unwrap_or(colors::TEXT_WHITE);
            result.push((item.text.clone(), left_x, text_y, color));
            left_x += item.text.len() as f32 * char_width + padding;
        }

        // Right items (render from right edge leftward)
        let mut right_x = zone.x + zone.width - padding;
        let mut right_items: Vec<&StatusItem> = items.iter()
            .filter(|i| i.alignment == StatusAlignment::Right)
            .collect();
        right_items.sort_by(|a, b| a.priority.cmp(&b.priority)); // Lowest priority = rightmost

        for item in &right_items {
            let text_width = item.text.len() as f32 * char_width;
            right_x -= text_width;
            let color = item.color.unwrap_or(colors::TEXT_WHITE);
            result.push((item.text.clone(), right_x, text_y, color));
            right_x -= padding;
        }

        result
    }
}

impl Default for StatusBar {
    fn default() -> Self {
        Self::new()
    }
}
