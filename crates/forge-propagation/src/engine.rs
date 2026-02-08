use std::collections::{HashMap, HashSet, VecDeque};
use crate::models::{FunctionId, PropagationResult, RippleNode};
use crate::graph::GraphStore;

pub struct PropagationEngine;

impl PropagationEngine {
    pub fn propagate(
        source: &FunctionId,
        delta: f64,
        graph: &GraphStore,
        current_scores: &HashMap<FunctionId, f64>,
    ) -> PropagationResult {
        let mut affected = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        // (FunctionId, current_delta, depth)
        queue.push_back((source.clone(), delta, 0));
        visited.insert(source.clone());

        while let Some((current_id, current_delta, depth)) = queue.pop_front() {
            if depth >= 3 {
                continue;
            }

            // Get dependents (incoming edges in dependency graph)
            // If A calls B (A -> B), then A depends on B.
            // If B changes (source), we need to update A.
            // So we traverse incoming edges to B.
            let dependents = graph.get_incoming_edges(&current_id);

            for (dep_id, edge_kind) in dependents {
                if visited.contains(&dep_id) {
                    continue;
                }

                let edge_weight = edge_kind.weight();
                let propagated_delta = current_delta * edge_weight;

                if propagated_delta.abs() < 0.01 {
                    continue;
                }

                let current_confidence = *current_scores.get(&dep_id).unwrap_or(&0.5);
                // Formula from spec: new_confidence = current_confidence * (1 - |propagated_delta|)
                // Wait, if delta is positive (improvement), confidence should increase?
                // The spec says: "new_confidence = current_confidence * (1 - |propagated_delta|)"
                // This implies ANY change ripples as a reduction in confidence (destabilization).
                // This makes sense for "Ripple Effect" — changing dependencies reduces confidence until re-verified.

                let new_confidence = current_confidence * (1.0 - propagated_delta.abs());

                affected.push(RippleNode {
                    function_id: dep_id.clone(),
                    file_path: dep_id.file.clone(),
                    delta: propagated_delta,
                    new_confidence,
                    relationship: edge_kind,
                    depth: depth + 1,
                });

                visited.insert(dep_id.clone());
                queue.push_back((dep_id, propagated_delta, depth + 1));
            }
        }

        affected.sort_by(|a, b| b.delta.abs().partial_cmp(&a.delta.abs()).unwrap());

        let total_files_affected = affected.iter().map(|n| &n.file_path).collect::<HashSet<_>>().len();
        let total_functions_affected = affected.len();

        PropagationResult {
            source: source.clone(),
            affected,
            total_files_affected,
            total_functions_affected,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::EdgeKind;

    #[test]
    fn propagation_attenuates() {
        // A -> B -> C (A calls B, B calls C).
        // Dependency graph: A -> B -> C.
        // If we modify C (source), B is affected, then A.
        // So we need C <- B <- A (incoming edges).

        let mut graph = GraphStore::new();
        let a = FunctionId { file: "a.ts".into(), name: "a".into(), line: 1 };
        let b = FunctionId { file: "b.ts".into(), name: "b".into(), line: 1 };
        let c = FunctionId { file: "c.ts".into(), name: "c".into(), line: 1 };

        graph.add_function(a.clone());
        graph.add_function(b.clone());
        graph.add_function(c.clone());

        // A calls B: A -> B
        graph.add_edge(&a, &b, EdgeKind::DirectCall).unwrap();
        // B calls C: B -> C
        graph.add_edge(&b, &c, EdgeKind::DirectCall).unwrap();

        let mut scores = HashMap::new();
        scores.insert(a.clone(), 0.95);
        scores.insert(b.clone(), 0.88);
        scores.insert(c.clone(), 0.82);

        // Modify C with delta -0.30
        let result = PropagationEngine::propagate(&c, -0.30, &graph, &scores);

        // Expect B affected (delta = -0.30 * 0.5 = -0.15)
        // Expect A affected (delta = -0.15 * 0.5 = -0.075)

        assert_eq!(result.affected.len(), 2);

        let b_node = result.affected.iter().find(|n| n.function_id == b).unwrap();
        assert!((b_node.delta - (-0.15)).abs() < 0.001);

        let a_node = result.affected.iter().find(|n| n.function_id == a).unwrap();
        assert!((a_node.delta - (-0.075)).abs() < 0.001);
    }
}
