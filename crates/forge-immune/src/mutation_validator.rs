use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationReport {
    pub total_mutants: usize,
    pub killed_mutants: usize,
    pub survived_mutants: usize,
    pub timeout_mutants: usize,
}

impl MutationReport {
    pub fn mutation_score(&self) -> f64 {
        if self.total_mutants == 0 {
            return 0.0;
        }
        // Assuming timeouts are considered kills (test failed to complete)
        (self.killed_mutants + self.timeout_mutants) as f64 / self.total_mutants as f64
    }
}

pub struct MutationValidator;

impl MutationValidator {
    /// Validates confidence based on mutation score.
    /// Returns a multiplier for the confidence score (0.0 to 1.0).
    pub fn validate(report: &MutationReport) -> f64 {
        report.mutation_score()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mutation_validator() {
        let report = MutationReport {
            total_mutants: 10,
            killed_mutants: 6,
            survived_mutants: 2,
            timeout_mutants: 2,
        };
        // killed (6) + timeout (2) = 8. Score = 0.8
        let factor = MutationValidator::validate(&report);
        assert!((factor - 0.8).abs() < 1e-6);
    }
}
