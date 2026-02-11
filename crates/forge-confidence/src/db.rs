use crate::models::ConfidenceScore;
use anyhow::Result;
use rusqlite::{params, Connection};
use std::path::Path;

pub struct ConfidenceDb {
    conn: Connection,
}

impl ConfidenceDb {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS confidence_scores (
                file_path TEXT NOT NULL,
                line INTEGER NOT NULL,
                score REAL NOT NULL CHECK(score >= 0.0 AND score <= 1.0),
                signals_json TEXT NOT NULL DEFAULT '[]',
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                PRIMARY KEY (file_path, line)
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_scores_file ON confidence_scores(file_path)",
            [],
        )?;

        Ok(Self { conn })
    }

    pub fn upsert_scores(&mut self, file: &str, scores: &[ConfidenceScore]) -> Result<()> {
        let tx = self.conn.transaction()?;

        {
            let mut stmt = tx.prepare(
                "INSERT OR REPLACE INTO confidence_scores (file_path, line, score, signals_json)
                 VALUES (?1, ?2, ?3, ?4)",
            )?;

            for score in scores {
                let signals_json = serde_json::to_string(&score.signals)?;
                stmt.execute(params![file, score.line, score.score, signals_json])?;
            }
        }

        tx.commit()?;
        Ok(())
    }

    pub fn get_scores(&self, file: &str) -> Result<Vec<ConfidenceScore>> {
        let mut stmt = self.conn.prepare(
            "SELECT line, score, signals_json FROM confidence_scores WHERE file_path = ?1",
        )?;

        let score_iter = stmt.query_map(params![file], |row| {
            let line: usize = row.get(0)?;
            let score: f64 = row.get(1)?;
            let signals_json: String = row.get(2)?;
            let signals = serde_json::from_str(&signals_json).unwrap_or_default();

            Ok(ConfidenceScore {
                line,
                score,
                signals,
            })
        })?;

        let mut scores = Vec::new();
        for score in score_iter {
            scores.push(score?);
        }

        Ok(scores)
    }

    pub fn delete_file(&self, file: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM confidence_scores WHERE file_path = ?1",
            params![file],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Signal, SignalKind};

    #[test]
    fn test_db_round_trip() {
        let mut db = ConfidenceDb::open(":memory:").unwrap();
        let scores = vec![ConfidenceScore::new(
            1,
            0.9,
            vec![Signal {
                name: SignalKind::SyntaxValid,
                value: 1.0,
                weight: 0.2,
                available: true,
            }],
        )];

        db.upsert_scores("test.rs", &scores).unwrap();
        let loaded = db.get_scores("test.rs").unwrap();

        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].line, 1);
        assert!((loaded[0].score - 0.9).abs() < 1e-10);
        assert_eq!(loaded[0].signals.len(), 1);
    }
}
