use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ExtensionInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub enabled: bool,
}

pub struct ExtensionsPanel {
    pub visible: bool,
    pub installed: Vec<ExtensionInfo>,
    pub search_query: String,
}

impl ExtensionsPanel {
    pub fn new() -> Self {
        Self {
            visible: false,
            installed: vec![
                // Placeholder extensions
                ExtensionInfo {
                    name: "Rust Analyzer".to_string(),
                    version: "0.3.1500".to_string(),
                    description: "Rust language support".to_string(),
                    enabled: true,
                },
                ExtensionInfo {
                    name: "Prettier".to_string(),
                    version: "2.8.0".to_string(),
                    description: "Code formatter".to_string(),
                    enabled: true,
                },
            ],
            search_query: String::new(),
        }
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    pub fn set_search_query(&mut self, query: String) {
        self.search_query = query;
    }

    pub fn toggle_extension(&mut self, name: &str) {
        if let Some(ext) = self.installed.iter_mut().find(|e| e.name == name) {
            ext.enabled = !ext.enabled;
        }
    }

    pub fn uninstall_extension(&mut self, name: &str) {
        if let Some(idx) = self.installed.iter().position(|e| e.name == name) {
            self.installed.remove(idx);
        }
    }
}
