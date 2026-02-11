use std::collections::{HashSet, VecDeque};
use crate::models::{FileNode, PropagationResult, RippleNode};
use crate::graph::GraphStore;
use crate::models::{DependencyEdge, DependencyKind, FileNode, PropagationResult, RippleNode};
use std::collections::{HashMap, HashSet, VecDeque};

pub struct PropagationEngine {
    pub damping: f64,
    pub max_depth: usize,
}

impl PropagationEngine {
    pub fn new() -> Self {
        Self {
            damping: 0.7,
            max_depth: 5,
        }
    }

    pub fn propagate(
        &self,
        source: &FileNode,
        delta: f64,
        graph: &GraphStore,
    ) -> PropagationResult {
        let mut affected = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back((source.path.clone(), delta, 0));
        visited.insert(source.path.clone());

        while let Some((current_path, current_delta, depth)) = queue.pop_front() {
            if depth >= self.max_depth {
                continue;
            }

            // Get dependents: incoming edges to current_path
            // If A imports B, A depends on B. B is source. A is dependent.
            let dependents = graph.get_dependents(&current_path);

            for (dep_node, edge) in dependents {
                if visited.contains(&dep_node.path) {
                    continue;
                }

                // Propagation formula: C(file_B) -= Δ × damping^distance
                // Distance here is 1 hop from current.
                // But we are accumulating damping.
                // Let's use recursive damping: next_delta = current_delta * edge_weight * damping
                let propagated_delta = current_delta * edge.weight * self.damping;

                // Threshold to stop negligible ripples
                if propagated_delta.abs() < 0.001 {
                    continue;
                }

                let current_confidence = dep_node.confidence;
                // New confidence reduces by propagated_delta
                // If delta is positive (source improved), dependents improve?
                // Spec says: C(file_B) -= Δ × damping^distance.
                // If Δ is +0.1 (improvement), C(B) decreases?
                // Wait. "Uncertainty Propagation".
                // If I change a file, I *introduce uncertainty*.
                // So any change (positive or negative delta in my head) might be Destabilizing.
                // But usually, if I fix a bug (confidence goes up), dependents should feel safer?
                // Or maybe the act of changing it makes dependents risky until re-verified.
                // The ticket says: "When C(file_A) changes by Δ, propagate: C(file_B) -= Δ × damping^distance"
                // If Δ is positive (confidence increased), we subtract it?
                // That would mean increasing confidence in A decreases confidence in B. That sounds wrong.
                // Unless Δ represents "Uncertainty". If Uncertainty increases, Confidence decreases.
                // But the input is "Confidence Score".
                // Let's assume Δ is change in Confidence.
                // If A improves (+0.1), B should improve (+something).
                // So C(B) += Δ × ...
                // But the formula says -=.
                // Maybe the "Δ" in the ticket refers to "Added Uncertainty"?
                // Let's stick to the interpretation:
                // "Ripple Effect" usually means *destabilization*.
                // When I modify A, even if I think it's better, B might break.
                // So any modification to A should arguably *lower* confidence in B until B is checked.
                // So: B_new = B_old - (|Δ| * factor).
                // This aligns with "Uncertainty Propagation".

                let destabilization = propagated_delta.abs();
                let new_confidence = (current_confidence - destabilization).clamp(0.0, 1.0);

                affected.push(RippleNode {
                    path: dep_node.path.clone(),
                    delta: -destabilization, // Net change is negative
                    new_confidence,
                    depth: depth + 1,
                });

                visited.insert(dep_node.path.clone());
                queue.push_back((dep_node.path, -destabilization, depth + 1));
            }
        }

        // Sort by impact magnitude
        affected.sort_by(|a, b| b.delta.abs().partial_cmp(&a.delta.abs()).unwrap());

        PropagationResult {
            source: source.clone(),
            affected,
        }
    }
}
