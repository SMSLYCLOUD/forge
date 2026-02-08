use serde::{Deserialize, Serialize};

/// UI modes that rearrange the layout for max effectiveness
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum UiMode {
    Standard,
    Focus,
    Performance,
    Debug,
    Zen,
    Review,
}

/// Layout configuration determined by the mode
#[derive(Clone, Debug)]
pub struct ModeLayoutConfig {
    pub activity_bar: bool,
    pub tab_bar: bool,
    pub breadcrumbs: bool,
    pub gutter: bool,
    pub status_bar: bool,
    pub ai_panel_allowed: bool,
    pub sidebar_allowed: bool,
    pub center_editor: bool,
    pub max_editor_width: Option<f32>,
    pub cursor_blink: bool,
    pub animations: bool,
    pub show_frame_time: bool,
}

impl UiMode {
    pub fn all() -> &'static [UiMode] {
        &[
            UiMode::Standard,
            UiMode::Focus,
            UiMode::Performance,
            UiMode::Debug,
            UiMode::Zen,
            UiMode::Review,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Standard => "🖥️ Standard",
            Self::Focus => "🎯 Focus",
            Self::Performance => "⚡ Perf",
            Self::Debug => "🐛 Debug",
            Self::Zen => "🧘 Zen",
            Self::Review => "📝 Review",
        }
    }

    pub fn shortcut(&self) -> &'static str {
        match self {
            Self::Standard => "Ctrl+Shift+1",
            Self::Focus => "Ctrl+Shift+F",
            Self::Performance => "Ctrl+Shift+H",
            Self::Debug => "F5",
            Self::Zen => "Ctrl+K Z",
            Self::Review => "Ctrl+Shift+R",
        }
    }

    pub fn layout_config(&self) -> ModeLayoutConfig {
        match self {
            Self::Standard => ModeLayoutConfig {
                activity_bar: true,
                tab_bar: true,
                breadcrumbs: true,
                gutter: true,
                status_bar: true,
                ai_panel_allowed: true,
                sidebar_allowed: true,
                center_editor: false,
                max_editor_width: None,
                cursor_blink: true,
                animations: true,
                show_frame_time: true,
            },
            Self::Focus => ModeLayoutConfig {
                activity_bar: false,
                tab_bar: true,
                breadcrumbs: false,
                gutter: true,
                status_bar: true,
                ai_panel_allowed: false,
                sidebar_allowed: false,
                center_editor: true,
                max_editor_width: Some(800.0),
                cursor_blink: true,
                animations: true,
                show_frame_time: false,
            },
            Self::Performance => ModeLayoutConfig {
                activity_bar: false,
                tab_bar: false,
                breadcrumbs: false,
                gutter: true,
                status_bar: true,
                ai_panel_allowed: false,
                sidebar_allowed: false,
                center_editor: false,
                max_editor_width: None,
                cursor_blink: false,
                animations: false,
                show_frame_time: true,
            },
            Self::Debug => ModeLayoutConfig {
                activity_bar: true,
                tab_bar: true,
                breadcrumbs: false,
                gutter: true,
                status_bar: true,
                ai_panel_allowed: false,
                sidebar_allowed: true,
                center_editor: false,
                max_editor_width: None,
                cursor_blink: true,
                animations: true,
                show_frame_time: true,
            },
            Self::Zen => ModeLayoutConfig {
                activity_bar: false,
                tab_bar: false,
                breadcrumbs: false,
                gutter: false,
                status_bar: false,
                ai_panel_allowed: false,
                sidebar_allowed: false,
                center_editor: true,
                max_editor_width: Some(700.0),
                cursor_blink: true,
                animations: false,
                show_frame_time: false,
            },
            Self::Review => ModeLayoutConfig {
                activity_bar: true,
                tab_bar: true,
                breadcrumbs: true,
                gutter: true,
                status_bar: true,
                ai_panel_allowed: true,
                sidebar_allowed: false,
                center_editor: false,
                max_editor_width: None,
                cursor_blink: true,
                animations: true,
                show_frame_time: false,
            },
        }
    }

    /// Cycle to next mode
    pub fn next(&self) -> Self {
        match self {
            Self::Standard => Self::Focus,
            Self::Focus => Self::Performance,
            Self::Performance => Self::Debug,
            Self::Debug => Self::Zen,
            Self::Zen => Self::Review,
            Self::Review => Self::Standard,
        }
    }

    /// Previous mode
    pub fn prev(&self) -> Self {
        match self {
            Self::Standard => Self::Review,
            Self::Focus => Self::Standard,
            Self::Performance => Self::Focus,
            Self::Debug => Self::Performance,
            Self::Zen => Self::Debug,
            Self::Review => Self::Zen,
        }
    }
}

impl Default for UiMode {
    fn default() -> Self {
        Self::Standard
    }
}
