use crate::{Encoding, History, LineEnding, Selection, Syntax, Transaction};
use anyhow::Result;
use ropey::Rope;
use std::path::Path;
use tree_sitter::Language;

/// The main text buffer with rope data structure and transaction-based editing
#[derive(Debug)]
pub struct Buffer {
    /// The text content stored as a rope
    rope: Rope,
    /// Undo/redo history tree
    history: History,
    /// Current selection state
    selection: Selection,
    /// Modified since last save
    dirty: bool,
    /// Original line ending style
    line_ending: LineEnding,
    /// Original file encoding
    encoding: Encoding,
    /// File path (if loaded from disk)
    path: Option<String>,
    /// Syntax highlighting state
    syntax: Option<Syntax>,
    /// Is the buffer fully loaded? (Async loading support)
    pub is_loading: bool,
}

// Manual Clone impl because Syntax is not Clone
impl Clone for Buffer {
    fn clone(&self) -> Self {
        Self {
            rope: self.rope.clone(),
            history: self.history.clone(),
            selection: self.selection.clone(),
            dirty: self.dirty,
            line_ending: self.line_ending,
            encoding: self.encoding,
            path: self.path.clone(),
            syntax: None, // We don't clone syntax state for now
            is_loading: self.is_loading,
        }
    }
}

#[cfg(test)]
mod edge_case_tests {
    use super::*;
    use crate::{Change, ChangeSet, Position, Transaction};

    #[test]
    fn empty_buffer_line_count() {
        let b = Buffer::new();
        assert_eq!(b.len_lines(), 1);
    }

    #[test]
    fn emoji_insert() {
        let mut b = Buffer::new();
        let change = Change::insert(Position::new(0), "👋🌍".to_string());
        let tx = Transaction::new(ChangeSet::with_change(change), None);
        b.apply(tx);
        assert!(b.text().contains("👋"));
    }

    #[test]
    fn cjk_insert() {
        let mut b = Buffer::new();
        let s = "你好世界";
        let change = Change::insert(Position::new(0), s.to_string());
        let tx = Transaction::new(ChangeSet::with_change(change), None);
        b.apply(tx);
        assert_eq!(b.text().len(), s.len());
    }

    #[test]
    fn zwj_sequence() {
        let mut b = Buffer::new();
        let s = "👨‍👩‍👧";
        let change = Change::insert(Position::new(0), s.to_string());
        let tx = Transaction::new(ChangeSet::with_change(change), None);
        b.apply(tx);
        assert!(b.text().contains("👨‍👩‍👧"));
    }

    #[test]
    fn large_buffer() {
        // Use fewer lines for speed in test, but enough to trigger rope chunks
        let text: String = (0..10_000).map(|i| format!("line {}\n", i)).collect();
        let b = Buffer::from_str(&text);
        assert_eq!(b.len_lines(), 10_001);
    }

    #[test]
    fn crlf_normalization() {
        let b = Buffer::from_str("hello\r\nworld");
        // We assert that it DOES contain CRLF because Buffer doesn't normalize by default
        assert!(b.text().contains("\r\n"));
    }
}

impl Buffer {
    /// Create a new empty buffer
    pub fn new() -> Self {
        Self {
            rope: Rope::new(),
            history: History::new(),
            selection: Selection::default(),
            dirty: false,
            line_ending: LineEnding::detect_system(),
            encoding: Encoding::Utf8,
            path: None,
            syntax: None,
            is_loading: false,
        }
    }

    /// Create a buffer from a string
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        Self {
            rope: Rope::from_str(s),
            history: History::new(),
            selection: Selection::default(),
            dirty: false,
            line_ending: LineEnding::detect_from_str(s),
            encoding: Encoding::Utf8,
            path: None,
            syntax: None,
            is_loading: false,
        }
    }

    /// Load a buffer from a file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())?;
        let line_ending = LineEnding::detect_from_str(&content);

        let mut buffer = Self {
            rope: Rope::from_str(&content),
            history: History::new(),
            selection: Selection::default(),
            dirty: false,
            line_ending,
            encoding: Encoding::Utf8,
            path: Some(path.as_ref().to_string_lossy().to_string()),
            syntax: None,
            is_loading: false,
        };

        // Auto-detect Rust
        if let Some(ext) = path.as_ref().extension() {
            if ext == "rs" {
                buffer.set_syntax(tree_sitter_rust::LANGUAGE.into());
            }
        }

        Ok(buffer)
    }

    /// Set the syntax language for the buffer
    pub fn set_syntax(&mut self, language: Language) {
        let mut syntax = Syntax::new(language);
        syntax.parse(&self.rope);
        self.syntax = Some(syntax);
    }

    /// Get the syntax state
    pub fn syntax(&self) -> Option<&Syntax> {
        self.syntax.as_ref()
    }

    /// Apply transaction and update syntax (internal helper)
    fn apply_transaction_internal(&mut self, transaction: &Transaction) {
        // Iterate changes
        for change in &transaction.changes.changes {
            // Update syntax BEFORE applying to rope
            if let Some(syntax) = &mut self.syntax {
                syntax.update(&self.rope, change);
            }

            // Apply to rope
            change.apply(&mut self.rope);
        }

        // Reparse syntax AFTER all changes to ensure consistency
        if let Some(syntax) = &mut self.syntax {
            syntax.reparse(&self.rope);
        }

        if let Some(new_selection) = &transaction.selection {
            self.selection = new_selection.clone();
        }
    }

    /// Apply a transaction to the buffer
    pub fn apply(&mut self, transaction: Transaction) {
        self.apply_transaction_internal(&transaction);

        // Add the ORIGINAL transaction to history (we'll invert on undo)
        self.history.push(transaction);
        self.dirty = true;
    }

    /// Undo the last transaction
    pub fn undo(&mut self) {
        if let Some(tx_to_undo) = self.history.get_current() {
            // Clone the transaction so we can use it after modifying history
            let tx_clone = tx_to_undo.clone();

            // Move back in history first
            if self.history.undo() {
                // Now invert and apply the transaction
                let inverse = tx_clone.invert(&self.rope);
                self.apply_transaction_internal(&inverse);
            }
        }
    }

    /// Redo the last undone transaction
    pub fn redo(&mut self) {
        if self.history.can_redo() {
            // Move forward in history
            if let Some(redo_tx) = self.history.redo().cloned() {
                // Apply the forward transaction
                self.apply_transaction_internal(&redo_tx);
            }
        }
    }

    /// Get the text content as a string
    pub fn text(&self) -> String {
        self.rope.to_string()
    }

    /// Get the underlying rope
    pub fn rope(&self) -> &Rope {
        &self.rope
    }

    /// Get a slice of the rope
    pub fn slice(&self, start: usize, end: usize) -> String {
        self.rope.slice(start..end).to_string()
    }

    /// Get the number of bytes in the buffer
    pub fn len_bytes(&self) -> usize {
        self.rope.len_bytes()
    }

    /// Get the number of lines in the buffer
    pub fn len_lines(&self) -> usize {
        self.rope.len_lines()
    }

    /// Get the buffer's selection
    pub fn selection(&self) -> &Selection {
        &self.selection
    }

    /// Set the buffer's selection
    pub fn set_selection(&mut self, selection: Selection) {
        self.selection = selection;
    }

    /// Add a range to the current selection (Multi-cursor)
    pub fn add_selection_range(&mut self, range: crate::Range) {
        self.selection.push(range);
    }

    /// Check if the buffer has been modified
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Mark the buffer as clean (after saving)
    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    /// Get the buffer's path
    pub fn path(&self) -> Option<&str> {
        self.path.as_deref()
    }

    /// Save the buffer to its file path
    pub fn save(&mut self) -> Result<()> {
        if let Some(ref path) = self.path {
            std::fs::write(path, self.text())?;
            self.mark_clean();
            Ok(())
        } else {
            Err(anyhow::anyhow!("No file path set"))
        }
    }

    /// Save the buffer to a specific path
    pub fn save_as<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        std::fs::write(path.as_ref(), self.text())?;
        self.path = Some(path.as_ref().to_string_lossy().to_string());
        self.mark_clean();
        Ok(())
    }

    /// Convert byte offset to line and column
    pub fn offset_to_line_col(&self, offset: usize) -> (usize, usize) {
        let line = self.rope.byte_to_line(offset);
        let line_start = self.rope.line_to_byte(line);
        let col = offset - line_start;
        (line, col)
    }

    /// Convert line and column to byte offset
    pub fn line_col_to_offset(&self, line: usize, col: usize) -> usize {
        let line_start = self.rope.line_to_byte(line);
        line_start + col
    }

    /// Sync content from another buffer (preserves selection/syntax state)
    pub fn sync_content_from(&mut self, other: &Buffer) {
        self.rope = other.rope.clone();
        self.history = other.history.clone();
        self.dirty = other.dirty;
        self.line_ending = other.line_ending;
        self.encoding = other.encoding;
        // Path should match, but we copy it anyway
        self.path = other.path.clone();
        // We don't sync syntax state as it depends on local parser state in Editor
        // But invalidating it might be good?
        // self.syntax = None;
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new()
    }
}

impl LineEnding {
    /// Detect the system's default line ending
    fn detect_system() -> Self {
        #[cfg(windows)]
        return LineEnding::CRLF;
        #[cfg(not(windows))]
        return LineEnding::LF;
    }

    /// Detect line ending from a string
    fn detect_from_str(s: &str) -> Self {
        if s.contains("\r\n") {
            LineEnding::CRLF
        } else if s.contains('\r') {
            LineEnding::CR
        } else {
            LineEnding::LF
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Change, ChangeSet, Position};

    #[test]
    fn test_buffer_creation() {
        let buffer = Buffer::new();
        assert_eq!(buffer.len_bytes(), 0);
        assert_eq!(buffer.len_lines(), 1); // Empty file has 1 line

        let buffer = Buffer::from_str("hello\nworld");
        assert_eq!(buffer.len_lines(), 2);
        assert_eq!(buffer.text(), "hello\nworld");
    }

    #[test]
    fn test_buffer_transactions() {
        let mut buffer = Buffer::from_str("hello world");

        // Replace "world" with "Forge"
        let change = Change::replace(Position::new(6), Position::new(11), "Forge".to_string());
        let tx = Transaction::new(ChangeSet::with_change(change), None);

        buffer.apply(tx);
        assert_eq!(buffer.text(), "hello Forge");
        assert!(buffer.is_dirty());
    }

    #[test]
    fn test_buffer_undo_redo() {
        let mut buffer = Buffer::from_str("original");

        let change = Change::insert(Position::new(8), " text".to_string());
        let tx = Transaction::new(ChangeSet::with_change(change), None);

        buffer.apply(tx);
        assert_eq!(buffer.text(), "original text");

        buffer.undo();
        assert_eq!(buffer.text(), "original");

        buffer.redo();
        assert_eq!(buffer.text(), "original text");
    }

    #[test]
    fn test_offset_to_line_col() {
        let buffer = Buffer::from_str("line1\nline2\nline3");

        let (line, col) = buffer.offset_to_line_col(0);
        assert_eq!((line, col), (0, 0));

        let (line, col) = buffer.offset_to_line_col(6); // Start of "line2"
        assert_eq!((line, col), (1, 0));

        let (line, col) = buffer.offset_to_line_col(8); // 'n' in "line2"
        assert_eq!((line, col), (1, 2));
    }

    #[test]
    fn test_syntax_integration() {
        // 1. Create buffer
        let mut buffer = Buffer::from_str("fn main() {}");

        // 2. Enable syntax (Rust)
        buffer.set_syntax(tree_sitter_rust::LANGUAGE.into());

        // 3. Verify initial tree
        {
            let syntax = buffer.syntax().expect("Syntax should be enabled");
            let tree = syntax.tree().expect("Tree should be parsed");
            let root = tree.root_node();
            assert_eq!(root.kind(), "source_file");
            // Check for function
            let func = root.child(0).expect("Should have a child");
            assert_eq!(func.kind(), "function_item");
        }

        // 4. Modify buffer (insert "pub ")
        let change = Change::insert(Position::new(0), "pub ".to_string());
        let tx = Transaction::new(ChangeSet::with_change(change), None);
        buffer.apply(tx);

        assert_eq!(buffer.text(), "pub fn main() {}");

        // 5. Verify updated tree
        {
            let syntax = buffer.syntax().expect("Syntax should be enabled");
            let tree = syntax.tree().expect("Tree should be parsed");
            let root = tree.root_node();
            let func = root.child(0).expect("Should have a child");
            assert_eq!(func.kind(), "function_item");
            // Check visibility modifier
            // The structure of `pub fn` might be `function_item` with a `visibility_modifier` child.
            let visibility = func.child(0).expect("Should have children");
            assert_eq!(visibility.kind(), "visibility_modifier");
        }

        // 6. Undo
        buffer.undo();
        assert_eq!(buffer.text(), "fn main() {}");

        // 7. Verify undo updated tree
        {
            let syntax = buffer.syntax().expect("Syntax should be enabled");
            let tree = syntax.tree().expect("Tree should be parsed");
            let root = tree.root_node();
            let func = root.child(0).expect("Should have a child");
            // Visibility modifier should be gone
            let first_child = func.child(0).expect("Should have children");
            assert_ne!(first_child.kind(), "visibility_modifier");
            assert_eq!(first_child.kind(), "fn");
        }
    }
}
