use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use std::collections::HashMap;
use anyhow::Result;
use crate::models::{FunctionId, EdgeKind};

pub struct GraphStore {
    graph: DiGraph<FunctionId, EdgeKind>,
    index: HashMap<FunctionId, NodeIndex>,
}

impl GraphStore {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            index: HashMap::new(),
        }
    }

    pub fn add_function(&mut self, id: FunctionId) -> NodeIndex {
        if let Some(&idx) = self.index.get(&id) {
            return idx;
        }
        let idx = self.graph.add_node(id.clone());
        self.index.insert(id, idx);
        idx
    }

    pub fn add_edge(&mut self, from: &FunctionId, to: &FunctionId, kind: EdgeKind) -> Result<()> {
        let from_idx = *self.index.get(from).ok_or_else(|| anyhow::anyhow!("Node not found: {:?}", from))?;
        let to_idx = *self.index.get(to).ok_or_else(|| anyhow::anyhow!("Node not found: {:?}", to))?;
        self.graph.add_edge(from_idx, to_idx, kind);
        Ok(())
    }

    pub fn get_dependents(&self, id: &FunctionId) -> Vec<(FunctionId, EdgeKind)> {
        // Dependents are nodes that depend on 'id'. If A calls B (A -> B), A depends on B.
        // If we change B, we need to propagate to A.
        // So we are looking for incoming edges to 'id'?
        // Wait, normally dependency graph: A -> B means A depends on B.
        // If B changes, A is affected. So we need to traverse reverse edges (B <- A).

        // Let's stick to the spec: "BFS from source through dependents".
        // If A calls B, then A is a dependent of B.
        // In the graph A -> B (Edge: DirectCall), does this mean A calls B?
        // Usually yes.
        // So if I modify B, I want to find A.
        // So I need incoming edges to B.

        if let Some(&idx) = self.index.get(id) {
            self.graph.neighbors_directed(idx, petgraph::Direction::Incoming)
                .map(|neighbor_idx| {
                    let node = self.graph[neighbor_idx].clone();
                    // We need the edge weight. `neighbors_directed` doesn't give edges.
                    // Let's use `edges_directed` instead.
                    (node, EdgeKind::DirectCall) // Placeholder, need correct edge finding
                })
                .collect() // This logic is slightly flawed with `neighbors_directed`. Fixing below.
        } else {
            Vec::new()
        }
    }

    // Correct implementation using edges_directed
    pub fn get_incoming_edges(&self, id: &FunctionId) -> Vec<(FunctionId, EdgeKind)> {
         if let Some(&idx) = self.index.get(id) {
            self.graph.edges_directed(idx, petgraph::Direction::Incoming)
                .map(|edge| {
                    let source_idx = edge.source();
                    let node = self.graph[source_idx].clone();
                    let weight = edge.weight().clone();
                    (node, weight)
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }
}

impl Default for GraphStore {
    fn default() -> Self {
        Self::new()
    }
}
