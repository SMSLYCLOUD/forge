mod tracker;
mod ema;
mod store;

pub use tracker::{ActionKind, DeveloperAction};
pub use ema::FeedbackEngine;
pub use store::FeedbackStore;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feedback_loop() {
        let mut engine = FeedbackEngine::new();
        // Module "safety.rs"
        let module = "safety.rs";

        // Initial trust
        assert_eq!(engine.get_prior(module), 0.5);

        // Developer ignores warnings (impact -0.1)
        // evidence = 0.5 - 0.1 = 0.4.
        // new = 0.1*0.4 + 0.9*0.5 = 0.04 + 0.45 = 0.49.
        engine.update(module, -0.1);
        assert!(engine.get_prior(module) < 0.5);

        // Developer keeps ignoring
        for _ in 0..50 {
            engine.update(module, -0.1);
        }

        // Should have low trust
        let final_prior = engine.get_prior(module);
        assert!(final_prior < 0.3);
    }

    #[test]
    fn test_persistence() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("feedback.json");

        let store = FeedbackStore {
            priors: [("test.rs".to_string(), 0.8)].into(),
            action_count: 5,
        };

        store.save(&path).unwrap();

        let loaded = FeedbackStore::load(&path).unwrap();
        assert_eq!(*loaded.priors.get("test.rs").unwrap(), 0.8);
        assert_eq!(loaded.action_count, 5);
    }
}
