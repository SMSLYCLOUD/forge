use crate::Transaction;
use std::time::{SystemTime, UNIX_EPOCH};

/// A node in the history tree
#[derive(Debug, Clone)]
pub struct HistoryNode {
    /// The transaction that was applied
    pub transaction: Transaction,
    /// Timestamp when this transaction was applied
    pub timestamp: u64,
    /// Parent node index (None for root)
    pub parent: Option<usize>,
    /// Child node indices
    pub children: Vec<usize>,
}

impl HistoryNode {
    fn new(transaction: Transaction, parent: Option<usize>) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            transaction,
            timestamp,
            parent,
            children: Vec::new(),
        }
    }
}

/// History tree for undo/redo. Unlike a linear undo stack, this preserves all history
/// even when you undo and make a different edit.
#[derive(Debug, Clone)]
pub struct History {
    /// All history nodes (tree stored as vector)
    pub nodes: Vec<HistoryNode>,
    /// Index of the current position in the history
    pub current: usize,
}

impl History {
    /// Create a new empty history
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            current: 0,
        }
    }

    /// Add a new transaction to the history
    pub fn push(&mut self, transaction: Transaction) -> usize {
        let parent = if self.nodes.is_empty() {
            None
        } else {
            Some(self.current)
        };

        let new_node = HistoryNode::new(transaction, parent);
        let new_idx = self.nodes.len();
        self.nodes.push(new_node);

        // Update parent's children
        if let Some(parent_idx) = parent {
            self.nodes[parent_idx].children.push(new_idx);
        }

        self.current = new_idx;
        new_idx
    }

    /// Undo: move back to the parent node
    pub fn undo(&mut self) -> bool {
        if self.nodes.is_empty() {
            return false;
        }

        let current_node = &self.nodes[self.current];
        if let Some(parent_idx) = current_node.parent {
            self.current = parent_idx;
            true
        } else {
            false
        }
    }

    /// Redo: move forward to a child node (uses the first child by default)
    pub fn redo(&mut self) -> Option<&Transaction> {
        if self.nodes.is_empty() {
            return None;
        }

        let current_node = &self.nodes[self.current];
        if let Some(&first_child) = current_node.children.first() {
            self.current = first_child;
            Some(&self.nodes[first_child].transaction)
        } else {
            None
        }
    }

    /// Get the current transaction
    pub fn current_transaction(&self) -> Option<&Transaction> {
        if self.nodes.is_empty() {
            None
        } else {
            Some(&self.nodes[self.current].transaction)
        }
    }

    /// Get the transaction at the current position (if any)
    pub fn get_current(&self) -> Option<&Transaction> {
        if self.nodes.is_empty() {
            None
        } else {
            Some(&self.nodes[self.current].transaction)
        }
    }

    /// Check if we can undo
    pub fn can_undo(&self) -> bool {
        !self.nodes.is_empty() && self.nodes[self.current].parent.is_some()
    }

    /// Check if we can redo
    pub fn can_redo(&self) -> bool {
        !self.nodes.is_empty() && !self.nodes[self.current].children.is_empty()
    }

    /// Get the total number of history nodes
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Check if history is empty
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

impl Default for History {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Change, ChangeSet, Position};

    #[test]
    fn test_history_push_and_undo() {
        let mut history = History::new();

        let tx1 = Transaction::new(
            ChangeSet::with_change(Change::insert(Position::new(0), "hello".to_string())),
            None,
        );
        let tx2 = Transaction::new(
            ChangeSet::with_change(Change::insert(Position::new(5), " world".to_string())),
            None,
        );

        history.push(tx1);
        history.push(tx2);

        assert_eq!(history.len(), 2);
        assert!(history.can_undo());

        history.undo();
        assert!(history.can_redo());

        history.redo();
        assert_eq!(history.current, 1);
    }

    #[test]
    fn test_history_branching() {
        let mut history = History::new();

        let tx1 = Transaction::new(
            ChangeSet::with_change(Change::insert(Position::new(0), "branch1".to_string())),
            None,
        );
        history.push(tx1);

        let tx2 = Transaction::new(
            ChangeSet::with_change(Change::insert(Position::new(7), " more".to_string())),
            None,
        );
        history.push(tx2);

        // Undo then make a different edit (creates a branch)
        history.undo();

        let tx3 = Transaction::new(
            ChangeSet::with_change(Change::insert(Position::new(7), " different".to_string())),
            None,
        );
        history.push(tx3);

        // Now node 0 has TWO children: node 1 and node 2
        assert_eq!(history.nodes[0].children.len(), 2);
    }
}
