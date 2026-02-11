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
