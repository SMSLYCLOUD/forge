use anyhow::Result;

/// Interface for the XGBoost Bug Predictor.
/// In production, this would load a trained .json/.ubj model and run inference.
/// For the MVP/Sandbox, we implement the feature extraction and a heuristic scoring function
/// that mimics the trained model behavior (Logic Regression).

pub struct BugPredictor {
    // model: xgboost::Booster,
}

#[derive(Debug, Default)]
pub struct CodeFeatures {
    pub cyclomatic_complexity: f64,
    pub ast_depth: f64,
    pub author_count: f64,
    pub churn_ratio: f64,
    pub loc: f64,
}

impl BugPredictor {
    pub fn new() -> Self {
        Self {}
    }

    /// Predict probability of a bug [0.0, 1.0]
    pub fn predict(&self, features: &CodeFeatures) -> Result<f64> {
        // Linear combination heuristic based on typical coefficients:
        // P(bug) = sigmoid(w0 + w1*CC + w2*Depth + w3*Authors + w4*Churn + w5*LOC)

        // Coefficients (hypothetical trained values)
        let w_cc = 0.05;
        let w_depth = 0.02;
        let w_authors = -0.1; // More authors might mean more review? Or opposite. Let's say -0.1 (Linus' law)
        let w_churn = 2.0;    // High churn is bad
        let w_loc = 0.001;
        let bias = -3.0;      // Baseline low probability

        let logit = bias
            + w_cc * features.cyclomatic_complexity
            + w_depth * features.ast_depth
            + w_authors * features.author_count
            + w_churn * features.churn_ratio
            + w_loc * features.loc;

        let prob = 1.0 / (1.0 + (-logit).exp());
        Ok(prob)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prediction_heuristic() {
        let predictor = BugPredictor::new();

        // Safe code
        let safe = CodeFeatures {
            cyclomatic_complexity: 1.0,
            ast_depth: 2.0,
            author_count: 5.0,
            churn_ratio: 0.0,
            loc: 50.0,
        };
        let p_safe = predictor.predict(&safe).unwrap();
        assert!(p_safe < 0.2); // Expect low bug prob

        // Risky code
        let risky = CodeFeatures {
            cyclomatic_complexity: 50.0, // Spaghetti
            ast_depth: 20.0,
            author_count: 1.0, // Solo dev
            churn_ratio: 0.8, // Rewritten recently
            loc: 2000.0, // Giant file
        };
        let p_risky = predictor.predict(&risky).unwrap();
        assert!(p_risky > 0.7); // Expect high bug prob
    }
}
