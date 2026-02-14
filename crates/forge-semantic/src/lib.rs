use anyhow::{Context, Result};
use rusqlite::{params, Connection, OpenFlags};
use serde::{Deserialize, Serialize};
use std::path::Path;
use uuid::Uuid;

// This is a placeholder for the ONNX embedding pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Embedding {
    pub vector: Vec<f32>,
}

pub struct EmbeddingEngine {
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

// ─── Vector DB Implementation ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub file_path: String,
    pub content: String,
    pub score: f32,
}

pub struct EmbeddingIndex {
    conn: Connection,
}

impl EmbeddingIndex {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS chunks (
                id TEXT PRIMARY KEY,
                file_path TEXT NOT NULL,
                content TEXT NOT NULL,
                vector BLOB NOT NULL
            )",
            [],
        )?;

        Ok(Self { conn })
    }

    pub fn insert(&self, file_path: &str, content: &str, embedding: &Embedding) -> Result<()> {
        let id = Uuid::new_v4().to_string();
        // Serialize vector as JSON for simple storage (real vector DBs use custom blob formats)
        // Or store as bytes (f32 LE)
        let vector_bytes: Vec<u8> = embedding
            .vector
            .iter()
            .flat_map(|f| f.to_le_bytes())
            .collect();

        self.conn.execute(
            "INSERT INTO chunks (id, file_path, content, vector) VALUES (?1, ?2, ?3, ?4)",
            params![id, file_path, content, vector_bytes],
        )?;
        Ok(())
    }

    pub fn search(&self, query_vector: &Embedding, limit: usize) -> Result<Vec<SearchResult>> {
        // Brute-force scan (SQLite doesn't natively support vector search without extensions like sqlite-vec)
        // For local dev with reasonable file counts, this is acceptable.
        let mut stmt = self.conn.prepare("SELECT id, file_path, content, vector FROM chunks")?;
        let rows = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let file_path: String = row.get(1)?;
            let content: String = row.get(2)?;
            let vector_blob: Vec<u8> = row.get(3)?;
            Ok((id, file_path, content, vector_blob))
        })?;

        let mut results = Vec::new();

        for row in rows {
            let (id, file_path, content, vector_blob) = row?;
            // Deserialize vector
            let vector: Vec<f32> = vector_blob
                .chunks_exact(4)
                .map(|chunk| {
                    let arr: [u8; 4] = chunk.try_into().unwrap();
                    f32::from_le_bytes(arr)
                })
                .collect();

            let candidate_emb = Embedding { vector };
            let score = EmbeddingEngine::cosine_similarity(query_vector, &candidate_emb);
            results.push(SearchResult {
                id,
                file_path,
                content,
                score,
            });
        }

        // Sort by score descending
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_insert_search() {
        let index = EmbeddingIndex::new(":memory:").unwrap();
        let engine = EmbeddingEngine::new("dummy").unwrap();

        let text1 = "rust memory safety";
        let emb1 = engine.embed(text1).unwrap();
        index.insert("doc1.rs", text1, &emb1).unwrap();

        let text2 = "python dynamic typing";
        let emb2 = engine.embed(text2).unwrap();
        index.insert("doc2.py", text2, &emb2).unwrap();

        // Search for something close to text1
        let results = index.search(&emb1, 1).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].content, text1);
        assert!(results[0].score > 0.99);
    }
}
