//! Forge Search - Search engine for Forge editor.

pub mod content;
pub mod fuzzy;

pub use content::{ContentSearcher, SearchOpts, SearchResult};
pub use fuzzy::{fuzzy_filter, fuzzy_score};
