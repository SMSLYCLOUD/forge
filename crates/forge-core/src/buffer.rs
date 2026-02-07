use crate::{Encoding, History, LineEnding, Position, Selection, Transaction};
use anyhow::Result;
use ropey::Rope;
use std::path::Path;

/// The main text buffer with rope data structure and transaction-based editing
#[derive(Debug, Clone)]
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
        }
    }

    /// Create a buffer from a string
    pub fn from_str(s: &str) -> Self {
        Self {
            rope: Rope::from_str(s),
            history: History::new(),
            selection: Selection::default(),
            dirty: false,
            line_ending: LineEnding::detect_from_str(s),
            encoding: Encoding::Utf8,
            path: None,
        }
    }

    /// Load a buffer from a file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())?;
        let line_ending = LineEnding::detect_from_str(&content);

        Ok(Self {
            rope: Rope::from_str(&content),
            history: History::new(),
            selection: Selection::default(),
            dirty: false,
            line_ending,
            encoding: Encoding::Utf8,
            path: Some(path.as_ref().to_string_lossy().to_string()),
        })
    }

    /// Apply a transaction to the buffer
    pub fn apply(&mut self, transaction: Transaction) {
        // Clone the transaction before we consume it
        let tx_for_history = transaction.clone();

        // Apply the transaction
        if let Some(new_selection) = transaction.apply(&mut self.rope) {
            self.selection = new_selection;
        }

        // Add the ORIGINAL transaction to history (we'll invert on undo)
        self.history.push(tx_for_history);
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
                inverse.apply(&mut self.rope);
            }
        }
    }

    /// Redo the last undone transaction
    pub fn redo(&mut self) {
        if self.history.can_redo() {
            // Move forward in history
            if let Some(redo_tx) = self.history.redo() {
                // Apply the forward transaction
                redo_tx.apply(&mut self.rope);
            }
        }
    }

    /// Get the text content as a string
    pub fn text(&self) -> String {
        self.rope.to_string()
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
    use crate::{Change, ChangeSet};

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
}
