use std::collections::HashMap;

// Exponential Moving Average
// prior_new = α × evidence + (1-α) × prior_old
const ALPHA: f64 = 0.1;

#[derive(Debug, Clone)]
pub struct FeedbackEngine {
    // Map module -> prior confidence (0.0 to 1.0)
    pub priors: HashMap<String, f64>,
    pub action_count: usize,
}

impl FeedbackEngine {
    pub fn new() -> Self {
        Self {
            priors: HashMap::new(),
            action_count: 0,
        }
    }

    pub fn update(&mut self, module: &str, evidence: f64) {
        let prior = self.priors.entry(module.to_string()).or_insert(0.5); // Start neutral

        // evidence is -1.0 to 1.0 (impact)
        // We need to map impact to a target confidence?
        // Or is the "evidence" the *change*?
        // Formula: prior_new = α × evidence + (1-α) × prior_old
        // If evidence is "FixFlaggedLine" (impact 0.2), does that mean evidence=0.2?
        // If prior is 0.5. new = 0.1*0.2 + 0.9*0.5 = 0.02 + 0.45 = 0.47.
        // Wait, fixing a line should INCREASE confidence (or developer trust).
        // If "FixFlaggedLine" means "The tool was right, I fixed it", then the tool's confidence was justified.
        // If "IgnoreWarning" means "The tool was wrong (false positive)", then the tool's confidence should drop.

        // Let's interpret `evidence` as "Target Confidence for this module based on this action".
        // If I fix a flagged line, I am validating the tool's concern.
        // But the prior here is "Developer Prior" or "Module Confidence"?
        // Ticket C7: "Developer actions feed back into confidence model... Per-module priors (developer might know module A well, not module B)"
        // "Store priors in local JSON... After ~50 actions, model is personalized"

        // Interpretation: This models the "Developer's Reliability" or "Trustworthiness of Code in this Module".
        // If I "IgnoreWarning" (impact -0.1), I am saying "I don't care about safety here".
        // So the system should trust this module LESS.
        // If I "AddTest" (impact 0.3), I am being responsible. Trust goes UP.

        // But the EMA formula requires `evidence` to be a value in the same domain as `prior` (0.0 to 1.0).
        // `impact` is a delta (-0.2, +0.3).
        // Let's treat `evidence` as `current_prior + impact`.
        // evidence = (current_prior + impact).clamp(0.0, 1.0)

        let target = (*prior + evidence).clamp(0.0, 1.0);
        *prior = ALPHA * target + (1.0 - ALPHA) * *prior;

        self.action_count += 1;
    }

    pub fn get_prior(&self, module: &str) -> f64 {
        *self.priors.get(module).unwrap_or(&0.5)
    }
}
