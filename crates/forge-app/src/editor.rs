//! Editor state — manages the text buffer, cursor, and viewport

use forge_core::{Buffer, Change, ChangeSet, Position, Selection, Transaction};
use forge_syntax::{HighlightSpan, Highlighter, Language, SyntaxParser};
use tracing::info;

/// The editor state: buffer + cursor logic + viewport
pub struct Editor {
    /// The text buffer (rope-backed)
    pub buffer: Buffer,
    /// Vertical scroll offset in lines
    pub scroll_y: f64,
    /// Whether the cursor should be visible (for blink)
    #[allow(dead_code)]
    pub cursor_visible: bool,
    /// Window title (derived from file path)
    pub title: String,
    /// Syntax parser for tree-sitter highlighting
    pub syntax_parser: Option<SyntaxParser>,
    /// Detected language
    pub language: Language,
    /// Cached highlight spans (byte-offset based)
    pub highlight_spans: Vec<HighlightSpan>,
}

impl Editor {
    /// Create an empty editor
    pub fn new() -> Self {
        Self {
            buffer: Buffer::new(),
            scroll_y: 0.0,
            cursor_visible: true,
            title: "Forge — [untitled]".to_string(),
            syntax_parser: None,
            language: Language::Unknown,
            highlight_spans: Vec::new(),
        }
    }

    /// Open a file in the editor
    pub fn open_file(path: &str) -> anyhow::Result<Self> {
        let buffer = Buffer::from_file(path)?;
        let filename = std::path::Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(path);
        info!("Opened: {} ({} lines)", filename, buffer.len_lines());

        let language = Language::from_path(path);
        let (syntax_parser, highlight_spans) = if language != Language::Unknown {
            if let Ok(mut parser) = SyntaxParser::new(language) {
                let text = buffer.text();
                if let Ok(tree) = parser.parse(&text) {
                    let spans = Highlighter::highlight(&tree, text.as_bytes(), language);
                    info!("Syntax: {:?} — {} highlight spans", language, spans.len());
                    (Some(parser), spans)
                } else {
                    (Some(parser), Vec::new())
                }
            } else {
                (None, Vec::new())
            }
        } else {
            (None, Vec::new())
        };

        Ok(Self {
            buffer,
            scroll_y: 0.0,
            cursor_visible: true,
            title: format!("Forge — {}", filename),
            syntax_parser,
            language,
            highlight_spans,
        })
    }

    /// Re-parse and re-highlight the buffer after edits
    pub fn rehighlight(&mut self) {
        if let Some(ref mut parser) = self.syntax_parser {
            let text = self.buffer.text();
            if let Ok(tree) = parser.parse(&text) {
                self.highlight_spans =
                    Highlighter::highlight(&tree, text.as_bytes(), self.language);
            }
        }
    }

    /// Get current scroll top line index
    pub fn scroll_top(&self) -> usize {
        self.scroll_y as usize
    }

    /// Set scroll top
    pub fn set_scroll_top(&mut self, line: usize) {
        self.scroll_y = line as f64;
    }

    /// Get total lines
    pub fn total_lines(&self) -> usize {
        self.buffer.len_lines()
    }

    /// Get the current cursor byte offset
    pub fn cursor_offset(&self) -> usize {
        self.buffer.selection().primary().head.into()
    }

    /// Get current cursor (line, col) — 0-indexed
    pub fn cursor_line_col(&self) -> (usize, usize) {
        self.buffer.offset_to_line_col(self.cursor_offset())
    }

    /// Get current cursor line (0-indexed)
    pub fn cursor_line(&self) -> usize {
        self.cursor_line_col().0
    }

    /// Get current cursor column (0-indexed)
    pub fn cursor_col(&self) -> usize {
        self.cursor_line_col().1
    }

    /// Insert a character at the cursor
    pub fn insert_char(&mut self, c: char) {
        let offset = self.cursor_offset();
        let s = c.to_string();

        let change = Change::insert(Position::new(offset), s.clone());
        let new_pos = Position::new(offset + s.len());
        let tx = Transaction::new(
            ChangeSet::with_change(change),
            Some(Selection::point(new_pos)),
        );
        self.buffer.apply(tx);
    }

    /// Insert a newline at cursor
    pub fn insert_newline(&mut self) {
        self.insert_char('\n');
    }

    /// Delete the character before the cursor (backspace)
    pub fn backspace(&mut self) {
        let offset = self.cursor_offset();
        if offset == 0 {
            return;
        }

        // Find the byte length of the previous character
        let text = self.buffer.text();
        let prev_char_len = text[..offset]
            .chars()
            .last()
            .map(|c| c.len_utf8())
            .unwrap_or(1);

        let start = offset - prev_char_len;
        let change = Change::delete(Position::new(start), Position::new(offset));
        let tx = Transaction::new(
            ChangeSet::with_change(change),
            Some(Selection::point(Position::new(start))),
        );
        self.buffer.apply(tx);
    }

    /// Delete the character after the cursor
    pub fn delete(&mut self) {
        let offset = self.cursor_offset();
        let len = self.buffer.len_bytes();
        if offset >= len {
            return;
        }

        let text = self.buffer.text();
        let next_char_len = text[offset..]
            .chars()
            .next()
            .map(|c| c.len_utf8())
            .unwrap_or(1);

        let end = offset + next_char_len;
        let change = Change::delete(Position::new(offset), Position::new(end));
        let tx = Transaction::new(
            ChangeSet::with_change(change),
            Some(Selection::point(Position::new(offset))),
        );
        self.buffer.apply(tx);
    }

    /// Move cursor left
    pub fn move_left(&mut self) {
        let offset = self.cursor_offset();
        if offset == 0 {
            return;
        }
        let text = self.buffer.text();
        let prev_len = text[..offset]
            .chars()
            .last()
            .map(|c| c.len_utf8())
            .unwrap_or(1);
        let new_offset = offset - prev_len;
        self.buffer
            .set_selection(Selection::point(Position::new(new_offset)));
    }

    /// Move cursor right
    pub fn move_right(&mut self) {
        let offset = self.cursor_offset();
        let len = self.buffer.len_bytes();
        if offset >= len {
            return;
        }
        let text = self.buffer.text();
        let next_len = text[offset..]
            .chars()
            .next()
            .map(|c| c.len_utf8())
            .unwrap_or(1);
        let new_offset = (offset + next_len).min(len);
        self.buffer
            .set_selection(Selection::point(Position::new(new_offset)));
    }

    /// Move cursor up one line
    pub fn move_up(&mut self) {
        let (line, col) = self.cursor_line_col();
        if line == 0 {
            return;
        }
        let new_offset = self.buffer.line_col_to_offset(line - 1, col);
        self.buffer
            .set_selection(Selection::point(Position::new(new_offset)));
    }

    /// Move cursor down one line
    pub fn move_down(&mut self) {
        let (line, col) = self.cursor_line_col();
        if line + 1 >= self.buffer.len_lines() {
            return;
        }
        let new_offset = self.buffer.line_col_to_offset(line + 1, col);
        self.buffer
            .set_selection(Selection::point(Position::new(new_offset)));
    }

    /// Move cursor to beginning of line
    pub fn move_home(&mut self) {
        let (line, _) = self.cursor_line_col();
        let new_offset = self.buffer.line_col_to_offset(line, 0);
        self.buffer
            .set_selection(Selection::point(Position::new(new_offset)));
    }

    /// Move cursor to end of line
    pub fn move_end(&mut self) {
        let (line, _) = self.cursor_line_col();
        // Get the line length by going to next line start and subtracting
        // Or simpler: buffer.line_len(line)
        // Assuming buffer has line_col_to_offset
        // Let's rely on line_col_to_offset(line + 1, 0) - 1 or similar logic
        // But safer is to iterate text.

        let line_start = self.buffer.line_col_to_offset(line, 0);
        let text = self.buffer.text();
        let line_slice = &text[line_start..];
        let line_len = line_slice.lines().next().unwrap_or("").len();

        let new_offset = line_start + line_len;
        self.buffer
            .set_selection(Selection::point(Position::new(new_offset)));
    }

    /// Scroll the viewport
    pub fn scroll(&mut self, delta: f64) {
        self.scroll_y = (self.scroll_y + delta).max(0.0);
        let max_scroll = (self.buffer.len_lines() as f64 - 1.0).max(0.0);
        self.scroll_y = self.scroll_y.min(max_scroll);
    }

    /// Ensure cursor is visible in viewport
    pub fn ensure_cursor_visible(&mut self, visible_lines: usize) {
        let (cursor_line, _) = self.cursor_line_col();
        let scroll_top = self.scroll_y as usize;
        let scroll_bottom = scroll_top + visible_lines.saturating_sub(1);

        if cursor_line < scroll_top {
            self.scroll_y = cursor_line as f64;
        } else if cursor_line > scroll_bottom {
            self.scroll_y = (cursor_line - visible_lines + 1) as f64;
        }
    }

    /// Save the file
    pub fn save(&mut self) -> anyhow::Result<()> {
        self.buffer.save()?;
        info!("Saved: {}", self.buffer.path().unwrap_or("[untitled]"));
        Ok(())
    }

    /// Update window title (adds * for dirty)
    pub fn window_title(&self) -> String {
        let base = &self.title;
        if self.buffer.is_dirty() {
            format!("● {}", base)
        } else {
            base.clone()
        }
    }

    /// Get all text for rendering
    #[allow(dead_code)]
    pub fn text(&self) -> String {
        self.buffer.text()
    }
}
