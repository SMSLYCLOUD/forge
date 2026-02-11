use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Extension manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ExtensionManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub icon: Option<String>,
}

/// Extension state
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum ExtensionState {
    Active,
    Disabled,
    Error(String),
}

/// Loaded extension
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Extension {
    pub manifest: ExtensionManifest,
    pub state: ExtensionState,
    pub install_path: PathBuf,
}

/// Extension registry
#[allow(dead_code)]
pub struct ExtensionRegistry {
    pub installed: Vec<Extension>,
    pub store_path: PathBuf,
}

impl ExtensionRegistry {
    #[allow(dead_code)]
    pub fn new() -> Self {
        let store_path = dirs_next::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".forge")
            .join("extensions");

        // Create directory if it doesn't exist
        let _ = std::fs::create_dir_all(&store_path);

        Self {
            installed: Vec::new(),
            store_path,
        }
    }

    /// Load installed extensions from disk
    #[allow(dead_code)]
    pub fn load_installed(&mut self) {
        self.installed.clear();

        if let Ok(entries) = std::fs::read_dir(&self.store_path) {
            for entry in entries.flatten() {
                let manifest_path = entry.path().join("forge-ext.toml");
                if manifest_path.exists() {
                    if let Ok(content) = std::fs::read_to_string(&manifest_path) {
                        if let Ok(manifest) = toml::from_str::<ExtensionManifest>(&content) {
                            self.installed.push(Extension {
                                manifest,
                                state: ExtensionState::Active,
                                install_path: entry.path(),
                            });
                        }
                    }
                }
            }
        }
    }

    /// Get installed extension count
    #[allow(dead_code)]
    pub fn count(&self) -> usize {
        self.installed.len()
    }

    /// List of built-in available extensions (hardcoded for now)
    #[allow(dead_code)]
    pub fn available() -> Vec<ExtensionManifest> {
        vec![
            ExtensionManifest {
                id: "forge-ext.word-count".into(),
                name: "Word Count".into(),
                version: "1.0.0".into(),
                author: "Forge".into(),
                description: "Displays word count in status bar".into(),
                icon: None,
            },
            ExtensionManifest {
                id: "forge-ext.bracket-pair".into(),
                name: "Bracket Pair Colorizer".into(),
                version: "1.0.0".into(),
                author: "Forge".into(),
                description: "Colors matching brackets".into(),
                icon: None,
            },
        ]
    }
}

impl Default for ExtensionRegistry {
    fn default() -> Self {
        Self::new()
    }
}
