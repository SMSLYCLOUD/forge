use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeveloperStats {
    pub commit_count: usize,
    pub bug_count: usize,
    pub review_acceptance_rate: f64, // 0.0 - 1.0
    pub last_edit: DateTime<Utc>,
    pub domain_expertise: f64, // 0.0 - 1.0
    pub flow_score: f64,       // 0.0 - 1.0
    pub fatigue_score: f64,    // 0.0 (fresh) - 1.0 (exhausted)
}

pub struct DeveloperModel;

impl DeveloperModel {
    pub fn compute_score(stats: &DeveloperStats, current_time: DateTime<Utc>) -> f64 {
        // 1. Base competence from history (cap at 100 commits)
        let history_score = (stats.commit_count as f64).min(100.0) / 100.0;

        // 2. Review quality
        let review_score = stats.review_acceptance_rate;

        // 3. Expertise
        let expertise = stats.domain_expertise;

        // 4. Flow
        let flow = stats.flow_score;

        // Weighted sum of positive signals
        // Weights sum to 1.0
        let base = 0.2 * history_score + 0.2 * review_score + 0.3 * expertise + 0.3 * flow;

        // 5. Recency (decay over 30 days)
        let age_days = (current_time - stats.last_edit).num_days().max(0) as f64;
        let recency_factor = (1.0 - age_days / 30.0).max(0.0);

        let with_recency = base * recency_factor;

        // 6. Penalty for bugs (e.g. 0.1 per bug, max 0.5)
        let bug_penalty = (stats.bug_count as f64 * 0.1).min(0.5);

        // 7. Fatigue (reduces confidence up to 0.5)
        let fatigue_penalty = stats.fatigue_score * 0.5;

        let final_score = with_recency - bug_penalty - fatigue_penalty;

        final_score.clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_developer_model() {
        let now = Utc::now();
        let stats = DeveloperStats {
            commit_count: 50, // 0.5
            bug_count: 0,
            review_acceptance_rate: 1.0, // 1.0
            last_edit: now,              // Factor 1.0
            domain_expertise: 0.8,       // 0.8
            flow_score: 0.8,             // 0.8
            fatigue_score: 0.0,
        };

        // Base = 0.2*0.5 + 0.2*1.0 + 0.3*0.8 + 0.3*0.8
        //      = 0.1 + 0.2 + 0.24 + 0.24 = 0.78
        // Recency = 1.0
        // Penalties = 0
        // Final = 0.78

        let score = DeveloperModel::compute_score(&stats, now);
        assert!((score - 0.78).abs() < 1e-6);

        // Test decay
        let old_stats = DeveloperStats {
            last_edit: now - Duration::days(15),
            ..stats.clone()
        };
        // Recency = 0.5
        // Score = 0.78 * 0.5 = 0.39
        let score_decay = DeveloperModel::compute_score(&old_stats, now);
        assert!((score_decay - 0.39).abs() < 1e-6);
    }
}
