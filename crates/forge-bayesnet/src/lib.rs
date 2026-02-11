use anyhow::Result;
use std::collections::HashMap;

/// A simple Bayesian Network engine for probability inference.
/// Uses variable elimination or sampling (Monte Carlo) for inference.
/// For MVP, we'll implement a simple node-based structure with parents and Conditional Probability Tables (CPTs).

#[derive(Debug, Clone)]
pub struct Node {
    pub name: String,
    pub parents: Vec<String>,
    // CPT: mapped by parent value combination -> probability of true
    // Key is vector of booleans representing parent states in order.
    // Value is P(node=true | parents)
    pub cpt: HashMap<Vec<bool>, f64>,
}

pub struct BayesNet {
    nodes: HashMap<String, Node>,
}

impl BayesNet {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    pub fn add_node(
        &mut self,
        name: &str,
        parents: &[&str],
        probabilities: HashMap<Vec<bool>, f64>,
    ) -> Result<()> {
        // Validate probabilities cover all 2^n parent combinations
        let expected_combinations = 1 << parents.len();
        if probabilities.len() != expected_combinations {
            return Err(anyhow::anyhow!(
                "Invalid CPT size for node {}. Expected {} entries, got {}.",
                name,
                expected_combinations,
                probabilities.len()
            ));
        }

        // Validate parents exist (topological sort enforcement not strictly required here but good practice)
        for p in parents {
            if !self.nodes.contains_key(*p) {
                return Err(anyhow::anyhow!("Parent node {} not found for {}", p, name));
            }
        }

        let node = Node {
            name: name.to_string(),
            parents: parents.iter().map(|s| s.to_string()).collect(),
            cpt: probabilities,
        };

        self.nodes.insert(name.to_string(), node);
        Ok(())
    }

    /// Infer probability P(target=true | evidence)
    /// Using simple Rejection Sampling for MVP (exact inference is NP-hard generally, though easy for small DAGs)
    pub fn infer(
        &self,
        target: &str,
        evidence: &HashMap<String, bool>,
        samples: usize,
    ) -> Result<f64> {
        if !self.nodes.contains_key(target) {
            return Err(anyhow::anyhow!("Target node {} not found", target));
        }

        let mut consistent_samples = 0;
        let mut true_samples = 0;

        let mut rng = rand::thread_rng();

        for _ in 0..samples {
            let sample = self.sample_network(&mut rng);

            // Check if sample matches evidence
            let mut matches_evidence = true;
            for (k, v) in evidence {
                if let Some(val) = sample.get(k) {
                    if val != v {
                        matches_evidence = false;
                        break;
                    }
                }
            }

            if matches_evidence {
                consistent_samples += 1;
                if *sample.get(target).unwrap() {
                    true_samples += 1;
                }
            }
        }

        if consistent_samples == 0 {
            return Ok(0.0); // Or undefined/fallback
        }

        Ok(true_samples as f64 / consistent_samples as f64)
    }

    fn sample_network(&self, rng: &mut rand::rngs::ThreadRng) -> HashMap<String, bool> {
        let mut values: HashMap<String, bool> = HashMap::new();
        // Naive topological sort by just iterating until all set (slow but works for small graphs)
        // Better: store topological order

        let _sorted_nodes: Vec<&Node> = self.nodes.values().collect();
        // This sorting is non-deterministic without proper topological sort logic.
        // Assuming user adds nodes in topological order for this MVP or we implement topological sort.
        // Let's implement a simple topological sort or just loop.

        // Correct approach: simple loop with max iterations
        let mut resolved = 0;
        while resolved < self.nodes.len() {
            let initial_resolved = resolved;
            for (name, node) in &self.nodes {
                if values.contains_key(name) {
                    continue;
                }

                let parents_ready = node.parents.iter().all(|p| values.contains_key(p));
                if parents_ready {
                    let parent_vals: Vec<bool> = node
                        .parents
                        .iter()
                        .map(|p| *values.get(p).unwrap())
                        .collect();

                    let prob_true = *node.cpt.get(&parent_vals).unwrap_or(&0.5);
                    let val = rand::Rng::gen_bool(rng, prob_true);
                    values.insert(name.clone(), val);
                    resolved += 1;
                }
            }
            if resolved == initial_resolved {
                // Cycle detected or disconnected?
                break;
            }
        }
        values
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rain_sprinkler_grass() {
        // Classic Bayes Net example
        // Rain -> GrassWet
        // Sprinkler -> GrassWet

        let mut net = BayesNet::new();

        // Rain: P(R=T) = 0.2
        let mut rain_cpt = HashMap::new();
        rain_cpt.insert(vec![], 0.2); // No parents
        net.add_node("Rain", &[], rain_cpt).unwrap();

        // Sprinkler: P(S=T | R=T) = 0.01, P(S=T | R=F) = 0.4
        let mut sprinkler_cpt = HashMap::new();
        sprinkler_cpt.insert(vec![true], 0.01);
        sprinkler_cpt.insert(vec![false], 0.40);
        net.add_node("Sprinkler", &["Rain"], sprinkler_cpt).unwrap();

        // GrassWet: P(G=T | S, R)
        let mut grass_cpt = HashMap::new();
        grass_cpt.insert(vec![false, false], 0.0); // No water
        grass_cpt.insert(vec![false, true], 0.8); // Rain only
        grass_cpt.insert(vec![true, false], 0.9); // Sprinkler only
        grass_cpt.insert(vec![true, true], 0.99); // Both
                                                  // Parent order: Sprinkler, Rain
        net.add_node("GrassWet", &["Sprinkler", "Rain"], grass_cpt)
            .unwrap();

        // Infer: P(Rain=T | GrassWet=T)
        // Analytical result is approx 0.3577
        let mut evidence = HashMap::new();
        evidence.insert("GrassWet".to_string(), true);

        let prob = net.infer("Rain", &evidence, 100_000).unwrap();
        println!("P(Rain | GrassWet) = {}", prob);

        // Sampling is approximate, check reasonable bounds
        assert!(prob > 0.30 && prob < 0.40);
    }
}
