use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileNode {
    pub path: String,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DependencyKind {
    Import,
    Call,
    Type,
    Inherit,
}

impl DependencyKind {
    pub fn weight(&self) -> f64 {
        match self {
            Self::Import => 0.30,
            Self::Call => 0.50,
            Self::Type => 0.40,
            Self::Inherit => 0.70,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyEdge {
    pub kind: DependencyKind,
    pub weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropagationResult {
    pub source: FileNode,
    pub affected: Vec<RippleNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RippleNode {
    pub path: String,
    pub delta: f64,
    pub new_confidence: f64,
    pub depth: usize,
}
