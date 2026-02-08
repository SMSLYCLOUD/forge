pub mod score;
pub mod color;
pub mod aggregate;
pub mod engine; // Legacy/Previous implementation to keep compatible if needed
pub mod models; // Legacy/Previous implementation
pub mod db; // Legacy/Previous implementation
pub mod temporal; // Legacy/Previous implementation

pub use score::{ConfidenceScore, CriteriaBreakdown, LineConfidence, EvidenceSource};
pub use color::RgbaColor;
pub use aggregate::aggregate_file_confidence;
