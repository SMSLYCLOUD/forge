/// Position in a text buffer. Can be represented as (line, column) or byte offset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Position {
    /// Byte offset from the start of the buffer
    pub offset: usize,
}

impl Position {
    pub fn new(offset: usize) -> Self {
        Self { offset }
    }

    pub fn zero() -> Self {
        Self { offset: 0 }
    }
}

impl From<usize> for Position {
    fn from(offset: usize) -> Self {
        Position::new(offset)
    }
}

impl From<Position> for usize {
    fn from(pos: Position) -> Self {
        pos.offset
    }
}
