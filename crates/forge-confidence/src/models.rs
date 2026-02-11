use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceScore {
    pub line: usize,
    pub score: f64,
    pub signals: Vec<Signal>,
}

impl ConfidenceScore {
    pub fn new(line: usize, score: f64, signals: Vec<Signal>) -> Self {
        Self {
            line,
            score: score.clamp(0.0, 1.0),
            signals,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signal {
    pub name: SignalKind,
    pub value: f64,
    pub weight: f64,
    pub available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SignalKind {
    SyntaxValid,
    TypeCheckPass,
    LintClean,
    TestCovers,
    TestPasses,
    MlBugProbability,
    CodeAgeFactor,
    AuthorExperience,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceUpdate {
    pub file_path: String,
    pub scores: Vec<ConfidenceScore>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confidence_clamps() {
        assert_eq!(ConfidenceScore::new(1, 1.5, vec![]).score, 1.0);
        assert_eq!(ConfidenceScore::new(1, -0.3, vec![]).score, 0.0);
    }
}
