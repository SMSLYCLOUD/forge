use crate::knowledge_graph::KnowledgeGraph;

pub struct BusFactor;

impl BusFactor {
    pub fn compute(graph: &KnowledgeGraph, module: &str, threshold: f64) -> usize {
        graph.get_developers_for_module(module)
            .into_iter()
            .filter(|(_, conf)| *conf >= threshold)
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bus_factor() {
        let mut graph = KnowledgeGraph::new();
        graph.set_confidence("dev1", "moduleA", 0.9);
        graph.set_confidence("dev2", "moduleA", 0.7);
        graph.set_confidence("dev3", "moduleA", 0.85);

        // Threshold 0.8
        // dev1 (0.9), dev3 (0.85). Total 2.

        let bf = BusFactor::compute(&graph, "moduleA", 0.8);
        assert_eq!(bf, 2);
    }
}
