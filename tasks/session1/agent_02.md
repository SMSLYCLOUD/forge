# Agent 02 — Implement forge-config + forge-keybindings

> **Read `tasks/GLOBAL_RULES.md` first.**

## Task A: Implement forge-config Properly

Replace the current stub `crates/forge-config/src/lib.rs` with a full TOML configuration system.

### `crates/forge-config/src/lib.rs`
```rust
//! Forge configuration — TOML-based with sensible defaults.
mod editor;
mod terminal;
pub use editor::EditorConfig;
pub use terminal::TerminalConfig;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ForgeConfig {
    pub editor: EditorConfig,
    pub terminal: TerminalConfig,
    pub theme: String,
    pub font_family: String,
    pub font_size: f32,
}

impl Default for ForgeConfig {
    fn default() -> Self {
        Self {
            editor: EditorConfig::default(),
            terminal: TerminalConfig::default(),
            theme: "Forge Dark".into(),
            font_family: "Cascadia Code".into(),
            font_size: 14.0,
        }
    }
}

impl ForgeConfig {
    pub fn load() -> Result<Self> {
        let path = Self::config_path();
        if path.exists() {
            let text = std::fs::read_to_string(&path)?;
            Ok(toml::from_str(&text)?)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, toml::to_string_pretty(self)?)?;
        Ok(())
    }

    pub fn config_path() -> PathBuf {
        dirs_next::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("forge")
            .join("config.toml")
    }
}
```

### `crates/forge-config/src/editor.rs`
```rust
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
            tab_size: 4, insert_spaces: true, word_wrap: false,
            line_numbers: true, minimap: true, auto_save_delay_ms: 30000,
            cursor_blink: true, bracket_matching: true, indent_guides: true,
        }
    }
}
```

### `crates/forge-config/src/terminal.rs`
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TerminalConfig {
    pub shell: Option<String>,
    pub scrollback: usize,
    pub cursor_style: String,
    pub font_size: f32,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self { shell: None, scrollback: 10000, cursor_style: "block".into(), font_size: 13.0 }
    }
}
```

Add tests for load/save round-trip using a temp directory.

## Task B: Add forge-keybindings Crate

### Create `crates/forge-keybindings/Cargo.toml`
```toml
[package]
name = "forge-keybindings"
version.workspace = true
edition.workspace = true
[dependencies]
serde = { workspace = true }
serde_json = { workspace = true }
anyhow = { workspace = true }
```

### `crates/forge-keybindings/src/lib.rs`
```rust
//! Keyboard shortcut system with configurable keybindings.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyCombo {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keybinding {
    pub key: KeyCombo,
    pub command: String,
    pub when: Option<String>,
}

pub struct KeybindingResolver {
    bindings: Vec<Keybinding>,
    index: HashMap<KeyCombo, Vec<usize>>,
}

impl KeybindingResolver {
    pub fn new(bindings: Vec<Keybinding>) -> Self {
        let mut index = HashMap::new();
        for (i, b) in bindings.iter().enumerate() {
            index.entry(b.key.clone()).or_insert_with(Vec::new).push(i);
        }
        Self { bindings, index }
    }

    pub fn resolve(&self, combo: &KeyCombo) -> Option<&str> {
        self.index.get(combo)
            .and_then(|indices| indices.last())
            .map(|&i| self.bindings[i].command.as_str())
    }

    pub fn default_keymap() -> Vec<Keybinding> {
        vec![
            Keybinding { key: KeyCombo { ctrl: true, shift: false, alt: false, key: "s".into() }, command: "file.save".into(), when: None },
            Keybinding { key: KeyCombo { ctrl: true, shift: false, alt: false, key: "z".into() }, command: "edit.undo".into(), when: None },
            Keybinding { key: KeyCombo { ctrl: true, shift: false, alt: false, key: "y".into() }, command: "edit.redo".into(), when: None },
            Keybinding { key: KeyCombo { ctrl: true, shift: false, alt: false, key: "f".into() }, command: "edit.find".into(), when: None },
            Keybinding { key: KeyCombo { ctrl: true, shift: false, alt: false, key: "h".into() }, command: "edit.replace".into(), when: None },
            Keybinding { key: KeyCombo { ctrl: true, shift: true, alt: false, key: "p".into() }, command: "command_palette".into(), when: None },
            Keybinding { key: KeyCombo { ctrl: true, shift: false, alt: false, key: "p".into() }, command: "file_picker".into(), when: None },
            Keybinding { key: KeyCombo { ctrl: true, shift: false, alt: false, key: "w".into() }, command: "tab.close".into(), when: None },
            Keybinding { key: KeyCombo { ctrl: true, shift: false, alt: false, key: "g".into() }, command: "go.line".into(), when: None },
            Keybinding { key: KeyCombo { ctrl: false, shift: false, alt: false, key: "F12".into() }, command: "go.definition".into(), when: None },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn resolve_ctrl_s() {
        let r = KeybindingResolver::new(KeybindingResolver::default_keymap());
        let combo = KeyCombo { ctrl: true, shift: false, alt: false, key: "s".into() };
        assert_eq!(r.resolve(&combo), Some("file.save"));
    }
    #[test]
    fn override_binding() {
        let mut bindings = KeybindingResolver::default_keymap();
        bindings.push(Keybinding { key: KeyCombo { ctrl: true, shift: false, alt: false, key: "s".into() }, command: "custom.save".into(), when: None });
        let r = KeybindingResolver::new(bindings);
        let combo = KeyCombo { ctrl: true, shift: false, alt: false, key: "s".into() };
        assert_eq!(r.resolve(&combo), Some("custom.save"));
    }
}
```

Add `"crates/forge-keybindings"` to workspace members.

**Acceptance**: `cargo test -p forge-config -p forge-keybindings` passes, 0 warnings.
