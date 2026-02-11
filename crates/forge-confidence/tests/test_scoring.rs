#[cfg(test)]
mod tests {
    use forge_confidence::aggregate::aggregate_file_confidence;
    use forge_confidence::color::{color_from_confidence, RgbaColor};
    use forge_confidence::score::{ConfidenceScore, CriteriaBreakdown, LineConfidence};

    #[test]
    fn test_color_gradient() {
        let red = color_from_confidence(0.0);
        assert_eq!(red, RgbaColor::new(255, 0, 0, 255));

        let yellow = color_from_confidence(0.5);
        assert_eq!(yellow, RgbaColor::new(255, 255, 0, 255));

        let green = color_from_confidence(1.0);
        assert_eq!(green, RgbaColor::new(0, 255, 0, 255));
    }

    #[test]
    fn test_cvar_aggregation() {
        // 100 lines. 95 lines perfect (1.0), 5 lines terrible (0.0).
        // Worst 5% = 5 lines. Average of 0.0 is 0.0.
        // Overall score should be 0.0 (risk sensitive).

        let mut lines = Vec::new();
        for i in 0..100 {
            let score = if i < 5 { 0.0 } else { 1.0 };
            lines.push(LineConfidence {
                line: i,
                score: ConfidenceScore {
                    overall: score,
                    criteria: CriteriaBreakdown {
                        syntax: score,
                        type_safety: score,
                        lint: score,
                        runtime: 0.5,
                        behavior: 0.5,
                        security: 0.5,
                    },
                    sources: vec![],
                },
                color: color_from_confidence(score),
            });
        }

        let agg = aggregate_file_confidence(&lines);
        assert!((agg.overall - 0.0).abs() < 1e-5);
    }
}
