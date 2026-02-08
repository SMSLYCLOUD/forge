use crate::score::{ConfidenceScore, LineConfidence};

pub fn aggregate_file_confidence(lines: &[LineConfidence]) -> ConfidenceScore {
    if lines.is_empty() {
        return ConfidenceScore::default();
    }

    // Sort scores to compute CVaR
    let mut scores: Vec<f64> = lines.iter().map(|l| l.score.overall).collect();
    scores.sort_by(|a, b| a.partial_cmp(b).unwrap());

    // CVaR at 95% confidence level means: expected value of the worst 5% of cases.
    // The spec says "Overall = CVaR_0.95".
    // Usually CVaR_alpha means average of the worst (1-alpha) tail.
    // So 5% tail.

    let tail_len = (scores.len() as f64 * 0.05).ceil().max(1.0) as usize;
    let worst_scores = &scores[0..tail_len];
    let avg_worst: f64 = worst_scores.iter().sum::<f64>() / tail_len as f64;

    // For detailed breakdown, we just average the whole file for now (simple approximation)
    // or we could do CVaR for each component. Let's do simple average for components to save complexity.
    let count = lines.len() as f64;
    let avg_syntax = lines.iter().map(|l| l.score.criteria.syntax).sum::<f64>() / count;
    let avg_type = lines.iter().map(|l| l.score.criteria.type_safety).sum::<f64>() / count;
    let avg_lint = lines.iter().map(|l| l.score.criteria.lint).sum::<f64>() / count;

    // Construct aggregated score
    // Note: overall is driven by risk (CVaR), components by average state.

    let mut agg = ConfidenceScore::default();
    agg.overall = avg_worst;
    agg.criteria.syntax = avg_syntax;
    agg.criteria.type_safety = avg_type;
    agg.criteria.lint = avg_lint;
    // ... others default

    agg
}
