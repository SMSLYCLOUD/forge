# TICKET C4 — ONNX Runtime Embedding Pipeline

## Context
- Source: Session 11 (Minimum RAM ML Architecture)
- Discovery: `discoveries/2026-02-08_sub-binary-ide-minimum-ram-ml.md`
- Use `ort` crate (ONNX Runtime for Rust), NOT Ollama
- Model: all-MiniLM-L6-v2 (22M params, 90MB, <5ms per embedding)

## Requirements
Build a Rust embedding service for code semantic similarity:

1. Load `all-MiniLM-L6-v2` ONNX model at startup
2. API: `embed(text: &str) -> Vec<f32>` (384-dim vector)
3. Batch API: `embed_batch(texts: &[&str]) -> Vec<Vec<f32>>`
4. HNSW vector index for nearest-neighbor search
5. Index: store embeddings for all functions/classes in project
6. Query: "find code similar to this function" → ranked results

### Performance Targets
- Single embedding: < 5ms
- Batch of 100: < 200ms
- Index lookup: < 1ms
- Memory: ~200MB total (model + index for 10K items)

## Files to Create
- `forge/crates/forge-semantic/src/lib.rs`
- `forge/crates/forge-semantic/src/embeddings.rs`
- `forge/crates/forge-semantic/src/index.rs` (HNSW)
- `forge/crates/forge-semantic/src/similarity.rs`
- `forge/crates/forge-semantic/Cargo.toml`
- `forge/crates/forge-semantic/tests/test_embeddings.rs`

## Acceptance Criteria
- [ ] ONNX model loads and runs
- [ ] Embeddings generated within 5ms
- [ ] HNSW index stores and retrieves nearest neighbors
- [ ] Similar code fragments return high scores
- [ ] Memory usage < 250MB
- [ ] Tests pass

## Effort: 3 days → WITH JULES: ~1 session
