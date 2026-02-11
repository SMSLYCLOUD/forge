#[derive(Debug, Clone, PartialEq)]
pub struct FilePicker {
    pub visible: bool,
    pub query: String,
    pub files: Vec<String>,
    pub filtered: Vec<(usize, f64)>, // index, score
}

impl Default for FilePicker {
    fn default() -> Self {
        Self {
            visible: false,
            query: String::new(),
            files: Vec::new(),
            filtered: Vec::new(),
        }
    }
}

impl FilePicker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(&mut self) {
        self.visible = true;
        self.query.clear();
        self.search("");
    }

    pub fn close(&mut self) {
        self.visible = false;
        self.query.clear();
        self.filtered.clear();
    }

    pub fn search(&mut self, query: &str) {
        self.query = query.to_string();

        if query.is_empty() {
            // Show all, no score (or 0.0)
            self.filtered = (0..self.files.len()).map(|i| (i, 0.0)).collect();
            return;
        }

        let mut scored = Vec::new();
        for (i, file) in self.files.iter().enumerate() {
            if let Some(score) = Self::fuzzy_score(query, file) {
                scored.push((i, score as f64));
            }
        }

        // Sort descending by score
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        self.filtered = scored;
    }

    pub fn select(&self, idx: usize) -> Option<&str> {
        if idx < self.filtered.len() {
            let file_idx = self.filtered[idx].0;
            self.files.get(file_idx).map(|s| s.as_str())
        } else {
            None
        }
    }

    fn fuzzy_score(query: &str, target: &str) -> Option<i32> {
        // Simple fuzzy match similar to CommandPalette
        let query_lower = query.to_lowercase();
        let target_lower = target.to_lowercase();

        let mut score = 0;
        let mut last_match_idx: Option<usize> = None;

        let target_chars: Vec<char> = target_lower.chars().collect();
        let mut target_cursor = 0;

        for qc in query_lower.chars() {
            let mut found = false;
            while target_cursor < target_chars.len() {
                let tc = target_chars[target_cursor];
                if tc == qc {
                    found = true;
                    score += target_chars.len() as i32 - target_cursor as i32;

                    if let Some(last) = last_match_idx {
                        if target_cursor == last + 1 {
                            score += 10;
                        } else {
                            score -= 1;
                        }
                    }

                    last_match_idx = Some(target_cursor);
                    target_cursor += 1;
                    break;
                }
                target_cursor += 1;
            }
            if !found {
                return None;
            }
        }
        Some(score)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search() {
        let mut fp = FilePicker::new();
        fp.files = vec![
            "src/main.rs".to_string(),
            "src/lib.rs".to_string(),
            "Cargo.toml".to_string(),
        ];

        fp.search("main");
        assert!(!fp.filtered.is_empty());
        assert_eq!(fp.select(0), Some("src/main.rs"));

        fp.search("toml");
        assert!(!fp.filtered.is_empty());
        assert_eq!(fp.select(0), Some("Cargo.toml"));
    }

    #[test]
    fn test_open_close() {
        let mut fp = FilePicker::new();
        fp.open();
        assert!(fp.visible);

        fp.close();
        assert!(!fp.visible);
        assert!(fp.filtered.is_empty());
    }
}
