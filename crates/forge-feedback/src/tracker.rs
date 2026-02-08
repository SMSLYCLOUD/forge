use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionKind {
    IgnoreWarning,
    FixFlaggedLine,
    DismissSuggestion,
    AddTest,
    CommitLowConfidence,
}

impl ActionKind {
    pub fn impact(&self) -> f64 {
        match self {
            Self::IgnoreWarning => -0.1,      // Negative reinforcement
            Self::FixFlaggedLine => 0.2,      // Positive reinforcement
            Self::DismissSuggestion => -0.05,
            Self::AddTest => 0.3,
            Self::CommitLowConfidence => -0.2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeveloperAction {
    pub kind: ActionKind,
    pub module: String,
    pub timestamp: i64,
}
