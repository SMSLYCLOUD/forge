use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum Key {
    Char(char),
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    Esc,
    Backspace,
    Enter,
    Space,
    Tab,
    Up, Down, Left, Right,
    Home, End, PageUp, PageDown,
    Delete,
    Backtick,
    Backslash,
    Slash,
    Plus,
    Minus,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub struct Modifiers {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub meta: bool,
}

impl Modifiers {
    pub const NONE: Self = Self { ctrl: false, shift: false, alt: false, meta: false };

    pub fn ctrl() -> Self { Self { ctrl: true, shift: false, alt: false, meta: false } }
    pub fn shift() -> Self { Self { ctrl: false, shift: true, alt: false, meta: false } }
    pub fn ctrl_shift() -> Self { Self { ctrl: true, shift: true, alt: false, meta: false } }
    pub fn alt() -> Self { Self { ctrl: false, shift: false, alt: true, meta: false } }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Keybinding {
    pub key: Key,
    pub mods: Modifiers,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Command {
    // File
    OpenFile, QuickOpen, NewFile, CloseTab, ReopenClosedTab, SwitchTab,
    // Editing
    Undo, Redo, Cut, Copy, Paste, SelectNextOccurence, DeleteLine, MoveLineUp, MoveLineDown,
    ToggleComment, Indent, Dedent, JumpToMatchingBracket,
    // Navigation
    GoToLine, GoToSymbol, GoToDefinition, FindReferences, NavigateBack, NavigateForward,
    GoToFileStart, GoToFileEnd,
    // Search
    Find, Replace, FindInFiles, NextMatch, PreviousMatch, CloseSearch,
    // View
    ToggleSidePanel, ToggleTerminal, SplitEditor, CommandPalette, ZoomIn, ZoomOut, ResetZoom,
}

pub struct Keymap {
    bindings: HashMap<Keybinding, Command>,
}

impl Keymap {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: Key, mods: Modifiers, command: Command) {
        self.bindings.insert(Keybinding { key, mods }, command);
    }

    pub fn get(&self, key: Key, mods: Modifiers) -> Option<&Command> {
        self.bindings.get(&Keybinding { key, mods })
    }

    /// Default VS Code-compatible keymap per spec
    pub fn default_vscode() -> Self {
        let mut map = Self::new();

        // File
        map.insert(Key::Char('o'), Modifiers::ctrl(), Command::OpenFile);
        map.insert(Key::Char('p'), Modifiers::ctrl(), Command::QuickOpen);
        map.insert(Key::Char('n'), Modifiers::ctrl(), Command::NewFile);
        map.insert(Key::Char('w'), Modifiers::ctrl(), Command::CloseTab);
        map.insert(Key::Tab, Modifiers::ctrl(), Command::SwitchTab);

        // Editing
        map.insert(Key::Char('z'), Modifiers::ctrl(), Command::Undo);
        map.insert(Key::Char('z'), Modifiers::ctrl_shift(), Command::Redo);
        map.insert(Key::Char('x'), Modifiers::ctrl(), Command::Cut);
        map.insert(Key::Char('c'), Modifiers::ctrl(), Command::Copy);
        map.insert(Key::Char('v'), Modifiers::ctrl(), Command::Paste);
        map.insert(Key::Char('d'), Modifiers::ctrl(), Command::SelectNextOccurence);
        map.insert(Key::Slash, Modifiers::ctrl(), Command::ToggleComment);

        // Navigation
        map.insert(Key::Char('g'), Modifiers::ctrl(), Command::GoToLine);
        map.insert(Key::F12, Modifiers::NONE, Command::GoToDefinition);

        // View
        map.insert(Key::Char('b'), Modifiers::ctrl(), Command::ToggleSidePanel);
        map.insert(Key::Backtick, Modifiers::ctrl(), Command::ToggleTerminal);
        map.insert(Key::Backslash, Modifiers::ctrl(), Command::SplitEditor);
        map.insert(Key::Char('p'), Modifiers::ctrl_shift(), Command::CommandPalette);

        map
    }
}
