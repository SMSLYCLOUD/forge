//! forge-core: Text buffer engine with rope data structure, transactions, and history tree.
//!
//! This is the heart of the Forge editor. Every text manipulation flows through this crate.

mod buffer;
mod history;
mod position;
mod selection;
mod transaction;

pub use buffer::Buffer;
pub use history::{History, HistoryNode};
pub use position::Position;
pub use selection::{Range, Selection};
pub use transaction::{Change, ChangeSet, Transaction};

/// Line ending styles
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineEnding {
    LF,   // Unix
    CRLF, // Windows
    CR,   // Old Mac
}

/// Character encoding
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Encoding {
    Utf8,
    Utf16Le,
    Utf16Be,
    Latin1,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
