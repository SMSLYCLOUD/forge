use serde::{Deserialize, Serialize};
use crate::color::RgbaColor;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConfidenceScore {
    pub overall: f64,           // 0.0 to 1.0
    pub criteria: CriteriaBreakdown,
    pub sources: Vec<EvidenceSource>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CriteriaBreakdown {
    pub syntax: f64,     // 1.0 if parseable (proven)
    pub type_safety: f64, // 1.0 if type-checked (proven)
    pub lint: f64,        // 1.0 if lint-clean (proven)
    pub runtime: f64,     // 0.0-1.0 (estimated)
    pub behavior: f64,    // 0.0-1.0 (estimated)
    pub security: f64,    // 0.0-1.0 (estimated)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EvidenceSource {
    TreeSitter,
    Lsp,
    Linter,
    TestRunner,
    SymbolicExecution,
    StaticAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineConfidence {
    pub line: usize,
    pub score: ConfidenceScore,
    pub color: RgbaColor,  // Gradient from red to green
}

impl Default for ConfidenceScore {
    fn default() -> Self {
        Self {
            overall: 0.5,
            criteria: CriteriaBreakdown {
                syntax: 0.0,
                type_safety: 0.0,
                lint: 0.0,
                runtime: 0.5,
                behavior: 0.5,
                security: 0.5,
            },
            sources: Vec::new(),
        }
    }
}
