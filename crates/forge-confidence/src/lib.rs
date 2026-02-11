pub mod aggregate;
pub mod color;
pub mod db; // Legacy/Previous implementation
pub mod engine; // Legacy/Previous implementation to keep compatible if needed
pub mod models; // Legacy/Previous implementation
pub mod score;
pub mod temporal; // Legacy/Previous implementation

pub use aggregate::aggregate_file_confidence;
pub use color::RgbaColor;
pub use score::{ConfidenceScore, CriteriaBreakdown, EvidenceSource, LineConfidence};
