use crate::{Position, Selection};
use ropey::Rope;

/// A single change in a transaction: delete range and/or insert text
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
    /// Start position of the change
    pub start: Position,
    /// End position of the change (if deleting)
    pub end: Position,
    /// Text to insert (if inserting)
    pub text: Option<String>,
}

impl Change {
    /// Create a deletion change
    pub fn delete(start: Position, end: Position) -> Self {
        Self {
            start,
            end,
            text: None,
        }
    }

    /// Create an insertion change
    pub fn insert(pos: Position, text: String) -> Self {
        Self {
            start: pos,
            end: pos,
            text: Some(text),
        }
    }

    /// Create a replacement change (delete + insert)
    pub fn replace(start: Position, end: Position, text: String) -> Self {
        Self {
            start,
            end,
            text: Some(text),
        }
    }

    /// Apply this change to a rope
    pub fn apply(&self, rope: &mut Rope) {
        // First remove the range if it's not empty
        if self.start != self.end {
            rope.remove(self.start.offset..self.end.offset);
        }

        // Then insert the text if any
        if let Some(ref text) = self.text {
            rope.insert(self.start.offset, text);
        }
    }

    /// Get the byte length change (negative for deletions, positive for insertions)
    pub fn len_delta(&self) -> isize {
        let deleted = (self.end.offset - self.start.offset) as isize;
        let inserted = self.text.as_ref().map_or(0, |t| t.len()) as isize;
        inserted - deleted
    }
}

/// A set of changes (for composing multiple changes)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeSet {
    pub changes: Vec<Change>,
}

impl ChangeSet {
    pub fn new() -> Self {
        Self {
            changes: Vec::new(),
        }
    }

    pub fn with_change(change: Change) -> Self {
        Self {
            changes: vec![change],
        }
    }

    pub fn add(&mut self, change: Change) {
        self.changes.push(change);
    }

    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }
}

impl Default for ChangeSet {
    fn default() -> Self {
        Self::new()
    }
}

/// An atomic, invertible transaction that can be applied to a buffer
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    /// The changes to apply
    pub changes: ChangeSet,
    /// The selection state after applying this transaction (optional)
    pub selection: Option<Selection>,
}

impl Transaction {
    /// Create a new transaction
    pub fn new(changes: ChangeSet, selection: Option<Selection>) -> Self {
        Self { changes, selection }
    }

    /// Create a transaction with a single change
    pub fn from_change(change: Change) -> Self {
        Self {
            changes: ChangeSet::with_change(change),
            selection: None,
        }
    }

    /// Apply this transaction to a rope, returning the selection if the transaction has one
    pub fn apply(&self, rope: &mut Rope) -> Option<Selection> {
        for change in &self.changes.changes {
            change.apply(rope);
        }
        self.selection.clone()
    }

    /// Create an inverted transaction that undoes this one
    pub fn invert(&self, rope: &Rope) -> Transaction {
        let mut inverted_changes = Vec::new();

        for change in self.changes.changes.iter().rev() {
            let inverted = if change.start == change.end {
                // Was an insertion, invert to deletion
                let end = Position::new(change.start.offset + change.text.as_ref().unwrap().len());
                Change::delete(change.start, end)
            } else if change.text.is_none() {
                // Was a deletion, invert to insertion
                let deleted_text = rope
                    .slice(change.start.offset..change.end.offset)
                    .to_string();
                Change::insert(change.start, deleted_text)
            } else {
                // Was a replacement, invert both parts
                let deleted_text = rope
                    .slice(change.start.offset..change.end.offset)
                    .to_string();
                let end = Position::new(change.start.offset + change.text.as_ref().unwrap().len());
                Change::replace(change.start, end, deleted_text)
            };
            inverted_changes.push(inverted);
        }

        Transaction {
            changes: ChangeSet {
                changes: inverted_changes,
            },
            selection: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_change_len_delta() {
        let insert = Change::insert(Position::new(0), "hello".to_string());
        assert_eq!(insert.len_delta(), 5);

        let delete = Change::delete(Position::new(0), Position::new(5));
        assert_eq!(delete.len_delta(), -5);

        let replace = Change::replace(Position::new(0), Position::new(5), "world!".to_string());
        assert_eq!(replace.len_delta(), 1); // -5 + 6 = 1
    }

    #[test]
    fn test_transaction_apply() {
        let mut rope = Rope::from_str("hello world");
        let change = Change::replace(Position::new(6), Position::new(11), "Forge".to_string());
        let tx = Transaction::from_change(change);

        tx.apply(&mut rope);
        assert_eq!(rope.to_string(), "hello Forge");
    }
}
