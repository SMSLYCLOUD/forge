use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeveloperId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ModuleId(pub String);

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct KnowledgeGraph {
    // We flatten the key for JSON serialization as arrays or custom map
    // Actually serde handles tuple keys in HashMap poorly for JSON (string keys only).
    // Better to use a Vec of structs or a custom serializer.
    // Or just format key as string "dev:module".
    // For simplicity, I'll use a Vec of edges for now, or just handle it in memory.
    // Wait, requirement says "All data local and opt-in".
    // I'll stick to HashMap but might need to adjust for Serde JSON if I want readable JSON.
    // Serde supports map with tuple keys but JSON requires string keys.
    // I'll use a string key map internally or a list of records.

    // For now, let's use a nested map: Developer -> Module -> Confidence
    // This is more natural for JSON.
    knowledge: HashMap<String, HashMap<String, f64>>,
}

impl KnowledgeGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_confidence(&mut self, dev: &str, module: &str, confidence: f64) {
        self.knowledge
            .entry(dev.to_string())
            .or_default()
            .insert(module.to_string(), confidence);
    }

    pub fn get_confidence(&self, dev: &str, module: &str) -> Option<f64> {
        self.knowledge.get(dev)?.get(module).copied()
    }

    pub fn get_developers_for_module(&self, module: &str) -> Vec<(String, f64)> {
        let mut result = Vec::new();
        for (dev, modules) in &self.knowledge {
            if let Some(&conf) = modules.get(module) {
                result.push((dev.clone(), conf));
            }
        }
        result
    }
}
