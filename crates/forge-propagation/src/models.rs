use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct FunctionId {
    pub file: String,
    pub name: String,
    pub line: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EdgeKind {
    DirectCall,
    TypeDependency,
    ImportSameModule,
    ImportCrossModule,
    TestCoverage,
    InterfaceImpl,
    Inheritance,
}

impl EdgeKind {
    pub fn weight(&self) -> f64 {
        match self {
            Self::DirectCall => 0.50,
            Self::TypeDependency => 0.40,
            Self::ImportSameModule => 0.30,
            Self::ImportCrossModule => 0.20,
            Self::TestCoverage => 0.80,
            Self::InterfaceImpl => 0.60,
            Self::Inheritance => 0.70,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropagationResult {
    pub source: FunctionId,
    pub affected: Vec<RippleNode>,
    pub total_files_affected: usize,
    pub total_functions_affected: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RippleNode {
    pub function_id: FunctionId,
    pub file_path: String,
    pub delta: f64,
    pub new_confidence: f64,
    pub relationship: EdgeKind,
    pub depth: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edge_weights() {
        assert_eq!(EdgeKind::DirectCall.weight(), 0.50);
        assert_eq!(EdgeKind::TestCoverage.weight(), 0.80);
    }
}
