use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use std::collections::HashMap;
use anyhow::Result;
use crate::models::{FileNode, DependencyKind, DependencyEdge};

pub struct GraphStore {
    pub graph: DiGraph<FileNode, DependencyEdge>,
    index: HashMap<String, NodeIndex>,
}

impl GraphStore {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            index: HashMap::new(),
        }
    }

    pub fn add_file(&mut self, path: String, confidence: f64) -> NodeIndex {
        if let Some(&idx) = self.index.get(&path) {
            // Update existing confidence
            self.graph[idx].confidence = confidence;
            return idx;
        }
        let node = FileNode { path: path.clone(), confidence };
        let idx = self.graph.add_node(node);
        self.index.insert(path, idx);
        idx
    }

    pub fn add_dependency(&mut self, from: &str, to: &str, kind: DependencyKind) -> Result<()> {
        let from_idx = *self.index.get(from).ok_or_else(|| anyhow::anyhow!("Node not found: {}", from))?;
        let to_idx = *self.index.get(to).ok_or_else(|| anyhow::anyhow!("Node not found: {}", to))?;

        let weight = kind.weight();
        self.graph.add_edge(from_idx, to_idx, DependencyEdge { kind, weight });
        Ok(())
    }

    pub fn get_dependents(&self, path: &str) -> Vec<(FileNode, DependencyEdge)> {
        // BFS propagation will use this.
        // If 'path' changes, we want to know what depends on it.
        // A -> B (A imports B). If B changes, A is affected.
        // So we need incoming edges to B.
        if let Some(&idx) = self.index.get(path) {
            self.graph.edges_directed(idx, petgraph::Direction::Incoming)
                .map(|edge| {
                    let source_idx = edge.source();
                    let node = self.graph[source_idx].clone();
                    let edge_data = edge.weight().clone();
                    (node, edge_data)
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn update_confidence(&mut self, path: &str, new_confidence: f64) -> Result<()> {
        if let Some(&idx) = self.index.get(path) {
            self.graph[idx].confidence = new_confidence;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Node not found: {}", path))
        }
    }
}

impl Default for GraphStore {
    fn default() -> Self {
        Self::new()
    }
}
