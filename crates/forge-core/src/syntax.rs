use crate::transaction::Change;
use ropey::Rope;
use tree_sitter::{InputEdit, Language, Parser, Point, Tree};

pub struct Syntax {
    parser: Parser,
    tree: Option<Tree>,
}

impl Syntax {
    pub fn new(language: Language) -> Self {
        let mut parser = Parser::new();
        parser
            .set_language(&language)
            .expect("Error loading language");

        Self { parser, tree: None }
    }

    /// Parse the entire buffer from scratch
    pub fn parse(&mut self, rope: &Rope) {
        self.tree = self.parser.parse_with(
            &mut |byte_offset, _| {
                if byte_offset >= rope.len_bytes() {
                    return &[] as &[u8];
                }
                let (chunk, chunk_byte_idx, _, _) = rope.chunk_at_byte(byte_offset);
                &chunk.as_bytes()[byte_offset - chunk_byte_idx..]
            },
            None,
        );
    }

    /// Update the syntax tree with a change
    /// Note: This updates the internal tree state but does not re-parse.
    /// You must call `reparse` after applying all changes.
    pub fn update(&mut self, rope: &Rope, change: &Change) {
        if let Some(tree) = &mut self.tree {
            let start_byte = change.start.offset;
            let old_end_byte = change.end.offset;
            let new_end_byte = start_byte + change.text.as_ref().map_or(0, |t| t.len());

            let start_position = position_at_byte(rope, start_byte);
            let old_end_position = position_at_byte(rope, old_end_byte);

            // To calculate new_end_position, we need to know how many lines/cols were added.
            // We can compute this by looking at the inserted text.
            let (new_lines, last_line_len) = if let Some(text) = &change.text {
                count_lines_cols(text)
            } else {
                (0, 0)
            };

            let new_end_row = start_position.row + new_lines;
            let new_end_column = if new_lines == 0 {
                start_position.column + last_line_len
            } else {
                last_line_len
            };

            let new_end_position = Point {
                row: new_end_row,
                column: new_end_column,
            };

            let edit = InputEdit {
                start_byte,
                old_end_byte,
                new_end_byte,
                start_position,
                old_end_position,
                new_end_position,
            };

            tree.edit(&edit);
        }
    }

    /// Re-parse the tree using the previous tree as a base
    pub fn reparse(&mut self, rope: &Rope) {
        self.tree = self.parser.parse_with(
            &mut |byte_offset, _| {
                if byte_offset >= rope.len_bytes() {
                    return &[] as &[u8];
                }
                let (chunk, chunk_byte_idx, _, _) = rope.chunk_at_byte(byte_offset);
                &chunk.as_bytes()[byte_offset - chunk_byte_idx..]
            },
            self.tree.as_ref(),
        );
    }

    pub fn tree(&self) -> Option<&Tree> {
        self.tree.as_ref()
    }
}

// Implement manual Debug for Parser wrapper if needed, but the struct derive works if Parser implements Debug
// If Parser doesn't implement Debug, we need:
/*
impl std::fmt::Debug for Syntax {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Syntax")
            .field("tree", &self.tree)
            .finish_non_exhaustive()
    }
}
*/
// The previous attempt showed Parser DOES NOT implement Debug. So removing derive and adding impl.

impl std::fmt::Debug for Syntax {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Syntax")
            .field("tree", &self.tree)
            .finish_non_exhaustive()
    }
}

fn position_at_byte(rope: &Rope, byte: usize) -> Point {
    let line = rope.byte_to_line(byte);
    let line_start = rope.line_to_byte(line);
    let col = byte - line_start;
    Point {
        row: line,
        column: col,
    }
}

fn count_lines_cols(text: &str) -> (usize, usize) {
    let mut lines = 0;
    let mut last_line_len = 0;
    for c in text.bytes() {
        if c == b'\n' {
            lines += 1;
            last_line_len = 0;
        } else {
            last_line_len += 1;
        }
    }
    (lines, last_line_len)
}
