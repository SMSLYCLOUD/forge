use crate::Position;
use smallvec::SmallVec;

/// A range in the text buffer with an anchor (immovable) and head (moving cursor).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Range {
    /// The immovable end of the selection
    pub anchor: Position,
    /// The moving end (where the cursor visually is)
    pub head: Position,
}

impl Range {
    pub fn new(anchor: Position, head: Position) -> Self {
        Self { anchor, head }
    }

    /// Create a point selection (cursor with no selection)
    pub fn point(pos: Position) -> Self {
        Self {
            anchor: pos,
            head: pos,
        }
    }

    /// Get the start position (min of anchor and head)
    pub fn start(&self) -> Position {
        self.anchor.min(self.head)
    }

    /// Get the end position (max of anchor and head)
    pub fn end(&self) -> Position {
        self.anchor.max(self.head)
    }

    /// Check if this is a point (cursor, not a selection)
    pub fn is_point(&self) -> bool {
        self.anchor == self.head
    }

    /// Get the length of the selection in bytes
    pub fn len(&self) -> usize {
        self.end().offset - self.start().offset
    }

    /// Check if the selection is empty (same as is_point but for API consistency)
    pub fn is_empty(&self) -> bool {
        self.is_point()
    }
}

/// A set of selections (can be multiple cursors/selections)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection {
    /// The selection ranges. Using SmallVec for single-cursor optimization.
    ranges: SmallVec<[Range; 1]>,
    /// Index of the primary selection
    primary_index: usize,
}

impl Selection {
    /// Create a new selection from ranges
    pub fn new(ranges: SmallVec<[Range; 1]>, primary_index: usize) -> Self {
        assert!(!ranges.is_empty(), "Selection must have at least one range");
        assert!(primary_index < ranges.len(), "Primary index out of bounds");
        Self {
            ranges,
            primary_index,
        }
    }

    /// Create a single-cursor selection at the given position
    pub fn point(pos: Position) -> Self {
        Self {
            ranges: smallvec::smallvec![Range::point(pos)],
            primary_index: 0,
        }
    }

    /// Create a single-range selection
    pub fn single(range: Range) -> Self {
        Self {
            ranges: smallvec::smallvec![range],
            primary_index: 0,
        }
    }

    /// Get all ranges
    pub fn ranges(&self) -> &[Range] {
        &self.ranges
    }

    /// Get the primary range
    pub fn primary(&self) -> &Range {
        &self.ranges[self.primary_index]
    }

    /// Get a mutable reference to the primary range
    pub fn primary_mut(&mut self) -> &mut Range {
        &mut self.ranges[self.primary_index]
    }

    /// Get the number of selections
    pub fn len(&self) -> usize {
        self.ranges.len()
    }

    /// Check if there are no selections (should never happen)
    pub fn is_empty(&self) -> bool {
        self.ranges.is_empty()
    }

    /// Map each range through a function
    pub fn map<F>(&self, f: F) -> Self
    where
        F: Fn(&Range) -> Range,
    {
        Self {
            ranges: self.ranges.iter().map(f).collect(),
            primary_index: self.primary_index,
        }
    }
}

impl Default for Selection {
    fn default() -> Self {
        Self::point(Position::zero())
    }
}
