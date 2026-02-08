use std::collections::HashMap;

pub struct MlCap;

impl MlCap {
    /// Caps the ML weight to ensure it doesn't exceed 25% of the total weight.
    /// Modifies the weights map in place.
    pub fn enforce_limit(weights: &mut HashMap<String, f64>, ml_key: &str) {
        let total_weight: f64 = weights.values().sum();
        if total_weight == 0.0 {
            return;
        }

        if let Some(&ml_weight) = weights.get(ml_key) {
            let limit = 0.25 * total_weight;
            if ml_weight > limit {
                // We need to reduce ml_weight so that new_ml_weight / new_total_weight <= 0.25
                // Let other_weight = total - ml.
                // We want ml / (other + ml) <= 0.25
                // ml <= 0.25 * other + 0.25 * ml
                // 0.75 * ml <= 0.25 * other
                // ml <= (1/3) * other

                let other_weight = total_weight - ml_weight;
                let new_ml_weight = other_weight / 3.0;

                weights.insert(ml_key.to_string(), new_ml_weight);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ml_cap() {
        let mut weights = HashMap::new();
        weights.insert("tests".to_string(), 0.5);
        weights.insert("reviews".to_string(), 0.5);
        weights.insert("ml".to_string(), 10.0); // Excessive ML weight

        // Total = 11.0. ML = 10.0. Ratio = 0.90

        MlCap::enforce_limit(&mut weights, "ml");

        let new_ml = *weights.get("ml").unwrap();
        let other = 1.0; // tests + reviews
        // new_ml should be other / 3 = 0.333...

        assert!((new_ml - 0.3333).abs() < 1e-4);

        let total = other + new_ml;
        assert!((new_ml / total - 0.25).abs() < 1e-4);
    }
}
