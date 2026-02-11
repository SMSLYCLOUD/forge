use forge_core::{Buffer, Position, Transaction, Change, ChangeSet};

pub struct RenameProvider;

impl RenameProvider {
    pub fn prepare_rename(buffer: &Buffer, pos: Position) -> Option<String> {
        // Find the word at the cursor position
        let (line, col) = buffer.offset_to_line_col(pos.offset);
        if line >= buffer.len_lines() {
            return None;
        }
        let line_content = buffer.rope().line(line).to_string();

        // Simple word detection logic (alphanumeric + underscore)
        // This is a placeholder, a real implementation would use tree-sitter or regex
        let chars: Vec<char> = line_content.chars().collect();
        if col >= chars.len() {
            return None;
        }

        if !chars[col].is_alphanumeric() && chars[col] != '_' {
            return None;
        }

        let mut start = col;
        while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
            start -= 1;
        }

        let mut end = col;
        while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
            end += 1;
        }

        Some(line_content[start..end].to_string())
    }

    pub fn apply_rename(
        buffer: &Buffer,
        pos: Position,
        new_name: &str,
    ) -> Option<Transaction> {
        let old_name = Self::prepare_rename(buffer, pos)?;

        // Find all occurrences of old_name
        // This is a naive implementation: string search
        // Real implementation should respect scope via tree-sitter/LSP
        let text = buffer.text();
        let mut changes = Vec::new();

        for (idx, _) in text.match_indices(&old_name) {
             changes.push(Change::replace(
                Position::new(idx),
                Position::new(idx + old_name.len()),
                new_name.to_string(),
            ));
        }

        if changes.is_empty() {
            return None;
        }

        Some(Transaction::new(ChangeSet { changes }, None))
    }
}
