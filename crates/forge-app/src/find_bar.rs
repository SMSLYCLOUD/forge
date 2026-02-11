use regex::RegexBuilder;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Match {
    pub line: usize,
    pub start_col: usize,
    pub end_col: usize,
}

pub struct FindBar {
    pub visible: bool,
    pub query: String,
    pub matches: Vec<Match>,
    pub current_match: Option<usize>,
    pub case_sensitive: bool,
    pub regex_mode: bool,
    pub whole_word: bool,
}

impl Default for FindBar {
    fn default() -> Self {
        Self {
            visible: false,
            query: String::new(),
            matches: Vec::new(),
            current_match: None,
            case_sensitive: false,
            regex_mode: false,
            whole_word: false,
        }
    }
}

impl FindBar {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(&mut self) {
        self.visible = true;
    }

    pub fn close(&mut self) {
        self.visible = false;
        self.query.clear();
        self.matches.clear();
        self.current_match = None;
    }

    pub fn set_case_sensitive(&mut self, value: bool) {
        self.case_sensitive = value;
    }

    pub fn set_regex(&mut self, value: bool) {
        self.regex_mode = value;
    }

    pub fn set_whole_word(&mut self, value: bool) {
        self.whole_word = value;
    }

    pub fn match_count(&self) -> usize {
        self.matches.len()
    }

    pub fn next_match(&mut self) -> Option<&Match> {
        if self.matches.is_empty() {
            return None;
        }

        if let Some(current) = self.current_match {
            if current + 1 < self.matches.len() {
                self.current_match = Some(current + 1);
            } else {
                self.current_match = Some(0);
            }
        } else {
            self.current_match = Some(0);
        }

        self.matches.get(self.current_match.unwrap())
    }

    pub fn prev_match(&mut self) -> Option<&Match> {
        if self.matches.is_empty() {
            return None;
        }

        if let Some(current) = self.current_match {
            if current > 0 {
                self.current_match = Some(current - 1);
            } else {
                self.current_match = Some(self.matches.len() - 1);
            }
        } else {
            self.current_match = Some(self.matches.len() - 1);
        }

        self.matches.get(self.current_match.unwrap())
    }

    pub fn search(&mut self, text: &str, query: &str) -> Vec<Match> {
        self.query = query.to_string();
        self.matches.clear();
        self.current_match = None;

        if query.is_empty() {
            return Vec::new();
        }

        if self.regex_mode {
            let mut builder = RegexBuilder::new(query);
            builder.case_insensitive(!self.case_sensitive);

            if let Ok(re) = builder.build() {
                for (line_idx, line) in text.lines().enumerate() {
                    for m in re.find_iter(line) {
                        self.matches.push(Match {
                            line: line_idx,
                            start_col: m.start(),
                            end_col: m.end(),
                        });
                    }
                }
            }
        } else {
            let query_lower = if self.case_sensitive {
                query.to_string()
            } else {
                query.to_lowercase()
            };

            for (line_idx, line) in text.lines().enumerate() {
                let line_to_search = if self.case_sensitive {
                    line.to_string()
                } else {
                    line.to_lowercase()
                };

                let mut start = 0;
                while let Some(idx) = line_to_search[start..].find(&query_lower) {
                    let absolute_idx = start + idx;
                    let end_idx = absolute_idx + query.len();

                    let is_match = if self.whole_word {
                        let pre_char = if absolute_idx > 0 {
                            line_to_search.chars().nth(absolute_idx - 1)
                        } else {
                            None
                        };
                        let post_char = line_to_search.chars().nth(end_idx);

                        let pre_ok = pre_char.map_or(true, |c| !c.is_alphanumeric());
                        let post_ok = post_char.map_or(true, |c| !c.is_alphanumeric());

                        pre_ok && post_ok
                    } else {
                        true
                    };

                    if is_match {
                        self.matches.push(Match {
                            line: line_idx,
                            start_col: absolute_idx,
                            end_col: end_idx,
                        });
                    }

                    start = absolute_idx + 1;
                }
            }
        }

        // Reset current match if we found some
        if !self.matches.is_empty() {
            self.current_match = Some(0);
        }

        self.matches.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_basic() {
        let mut bar = FindBar::new();
        let text = "hello world\nhello universe";
        let matches = bar.search(text, "hello");

        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].line, 0);
        assert_eq!(matches[0].start_col, 0);
        assert_eq!(matches[1].line, 1);
        assert_eq!(matches[1].start_col, 0);
    }

    #[test]
    fn test_next_prev_match() {
        let mut bar = FindBar::new();
        let text = "a\na\na";
        bar.search(text, "a");

        assert_eq!(bar.match_count(), 3);

        // Initial state is first match (index 0)
        assert_eq!(bar.current_match, Some(0));

        // Next -> 1
        let m = bar.next_match();
        assert!(m.is_some());
        assert_eq!(bar.current_match, Some(1));

        // Next -> 2
        let m = bar.next_match();
        assert!(m.is_some());
        assert_eq!(bar.current_match, Some(2));

        // Next -> 0 (cycle)
        let m = bar.next_match();
        assert!(m.is_some());
        assert_eq!(bar.current_match, Some(0));

        // Prev -> 2 (cycle back)
        let m = bar.prev_match();
        assert!(m.is_some());
        assert_eq!(bar.current_match, Some(2));
    }

    #[test]
    fn test_case_sensitivity() {
        let mut bar = FindBar::new();
        let text = "Hello hello";

        bar.set_case_sensitive(true);
        let matches = bar.search(text, "Hello");
        assert_eq!(matches.len(), 1);

        bar.set_case_sensitive(false);
        let matches = bar.search(text, "Hello");
        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn test_whole_word() {
        let mut bar = FindBar::new();
        let text = "hello helloworld hello";

        bar.set_whole_word(true);
        let matches = bar.search(text, "hello");
        assert_eq!(matches.len(), 2); // First and last

        bar.set_whole_word(false);
        let matches = bar.search(text, "hello");
        assert_eq!(matches.len(), 3); // All occurrences
    }

    #[test]
    fn test_regex() {
        let mut bar = FindBar::new();
        let text = "abc 123 def 456";

        bar.set_regex(true);
        let matches = bar.search(text, "\\d+");
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].start_col, 4);
        assert_eq!(matches[1].start_col, 12);
    }
}
