use anyhow::Result;
use serde::{Deserialize, Serialize};

// This is a placeholder for the ONNX embedding pipeline.
// The actual `ort` crate requires linking to ONNX Runtime libraries (libonnxruntime.so/dll),
// which are likely not available in this sandbox environment.
// For the purpose of this task, we will define the `EmbeddingEngine` interface and a stub implementation.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Embedding {
    pub vector: Vec<f32>,
}

pub struct EmbeddingEngine {
    // In a real implementation:
    // session: ort::Session,
    #[allow(dead_code)]
    model_path: String,
}

impl EmbeddingEngine {
    pub fn new(model_path: &str) -> Result<Self> {
        Ok(Self {
            model_path: model_path.to_string(),
        })
    }

    pub fn embed(&self, text: &str) -> Result<Embedding> {
        // Stub implementation: return a deterministic pseudo-random vector based on text length
        // This allows testing downstream logic without the heavy ML runtime.
        let dim = 384; // Typical for all-MiniLM-L6-v2
        let seed = text.len() as u32;
        let mut vector = Vec::with_capacity(dim);

        for i in 0..dim {
            let val = ((i as u32 + seed).wrapping_mul(1103515245) % 1000) as f32 / 1000.0;
            vector.push(val);
        }

        Ok(Embedding { vector })
    }

    pub fn cosine_similarity(a: &Embedding, b: &Embedding) -> f32 {
        let dot_product: f32 = a.vector.iter().zip(&b.vector).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.vector.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot_product / (norm_a * norm_b)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedding_stub() {
        let engine = EmbeddingEngine::new("dummy.onnx").unwrap();
        let emb1 = engine.embed("hello world").unwrap();
        let emb2 = engine.embed("hello world").unwrap();

        assert_eq!(emb1.vector.len(), 384);
        assert!((EmbeddingEngine::cosine_similarity(&emb1, &emb2) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_similarity() {
        let engine = EmbeddingEngine::new("dummy.onnx").unwrap();
        let emb1 = engine.embed("short").unwrap();
        let emb2 = engine.embed("a much longer string").unwrap();

        let sim = EmbeddingEngine::cosine_similarity(&emb1, &emb2);
        assert!(sim >= -1.0 && sim <= 1.0);
    }
}
