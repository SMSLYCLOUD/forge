use forge_immune::{MutationValidator, MutationReport, AnomalyDetector, ActionType, MlCap, TemporalDecay, AuditLog};
use std::collections::HashMap;
use chrono::{Utc, Duration};

#[test]
fn test_immune_components_integration() {
    // 1. Mutation Validator
    let report = MutationReport {
        total_mutants: 20,
        killed_mutants: 15,
        survived_mutants: 5,
        timeout_mutants: 0,
    };
    let score = MutationValidator::validate(&report);
    assert_eq!(score, 0.75);

    // 2. Anomaly Detector
    let mut detector = AnomalyDetector::new(5);
    // Add 2 safe actions
    detector.record_action(ActionType::FixWarning);
    detector.record_action(ActionType::FixWarning);
    // Add 2 dismissals
    detector.record_action(ActionType::DismissWarning);
    detector.record_action(ActionType::DismissWarning);
    // History: [F, F, D, D]. Len 4. Rate 0.5. Not > 0.8.
    assert!(!detector.is_anomalous());

    // Add 1 more dismissal.
    detector.record_action(ActionType::DismissWarning);
    // History: [F, F, D, D, D]. Len 5. Rate 0.6. Not > 0.8.
    assert!(!detector.is_anomalous());

    // Fill with dismissals to push out Fixes.
    detector.record_action(ActionType::DismissWarning); // Pushes out F. [F, D, D, D, D]. Rate 0.8. Not > 0.8.
    detector.record_action(ActionType::DismissWarning); // Pushes out F. [D, D, D, D, D]. Rate 1.0. > 0.8.

    assert!(detector.is_anomalous());

    // 3. ML Cap
    let mut weights = HashMap::new();
    weights.insert("ml".to_string(), 0.5);
    weights.insert("other".to_string(), 0.5); // total 1.0. ML 50%.
    MlCap::enforce_limit(&mut weights, "ml");
    let ml = weights["ml"];
    // other is still 0.5.
    // ml <= 0.25 * (ml + other) -> ml <= 1/3 * other -> ml <= 0.1666...
    assert!((ml - 0.1666).abs() < 1e-3);

    // 4. Temporal Decay
    let decay = TemporalDecay::new(30);
    let now = Utc::now();
    let old = now - Duration::days(60);
    assert_eq!(decay.apply(1.0, old), 0.0);

    // 5. Audit Log
    let mut log = AuditLog::new();
    log.append("User verified mutation score".to_string());
    log.append("System capped ML weight".to_string());
    assert!(log.verify());
}
