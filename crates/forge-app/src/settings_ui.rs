#[derive(Clone, Debug)]
pub enum SettingValue {
    Bool(bool),
    String(String),
    Number(f64),
    Enum(Vec<String>, usize),
}

#[derive(Clone, Debug)]
pub struct SettingEntry {
    pub key: String,
    pub label: String,
    pub value: SettingValue,
    pub description: String,
}

#[derive(Clone, Debug)]
pub struct Category {
    pub name: String,
    pub settings: Vec<SettingEntry>,
}

pub struct SettingsUi {
    pub visible: bool,
    pub search_query: String,
    pub categories: Vec<Category>,
    pub active_category: usize,
}

impl SettingsUi {
    pub fn new() -> Self {
        Self {
            visible: false,
            search_query: String::new(),
            categories: vec![
                Category {
                    name: "Editor".to_string(),
                    settings: vec![
                        SettingEntry {
                            key: "editor.fontSize".to_string(),
                            label: "Font Size".to_string(),
                            value: SettingValue::Number(14.0),
                            description: "Controls the font size in pixels.".to_string(),
                        },
                        SettingEntry {
                            key: "editor.wordWrap".to_string(),
                            label: "Word Wrap".to_string(),
                            value: SettingValue::Bool(false),
                            description: "Controls how lines should wrap.".to_string(),
                        },
                    ],
                },
                Category {
                    name: "Files".to_string(),
                    settings: vec![SettingEntry {
                        key: "files.autoSave".to_string(),
                        label: "Auto Save".to_string(),
                        value: SettingValue::Enum(
                            vec![
                                "off".to_string(),
                                "afterDelay".to_string(),
                                "onFocusChange".to_string(),
                            ],
                            0,
                        ),
                        description: "Controls auto save behavior.".to_string(),
                    }],
                },
            ],
            active_category: 0,
        }
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    pub fn set_search_query(&mut self, query: String) {
        self.search_query = query;
        // Search logic could filter categories/settings here
    }

    pub fn select_category(&mut self, index: usize) {
        if index < self.categories.len() {
            self.active_category = index;
        }
    }
}
