use anyhow::{Context, Result};
use forge_search::{ContentSearcher, SearchOpts, SearchResult};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

pub struct SearchPanel {
    pub visible: bool,
    pub query: String,
    pub replace_query: String,
    pub regex: bool,
    pub case_sensitive: bool,
    pub whole_word: bool,
    pub include_glob: String,
    pub exclude_glob: String,

    pub results: Vec<SearchResult>,
    pub selected_index: Option<usize>,

    // Async search handling
    tx: Sender<Vec<SearchResult>>,
    rx: Receiver<Vec<SearchResult>>,
    pub searching: bool,

    // UI renderer state
    pub ui: crate::search_results_ui::SearchResultsUi,
}

impl Default for SearchPanel {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            visible: false,
            query: String::new(),
            replace_query: String::new(),
            regex: false,
            case_sensitive: false,
            whole_word: false,
            include_glob: String::new(),
            exclude_glob: String::new(),
            results: Vec::new(),
            selected_index: None,
            tx,
            rx,
            searching: false,
            ui: crate::search_results_ui::SearchResultsUi::new(),
        }
    }
}

impl SearchPanel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn search(&mut self, root: &Path) {
        if self.query.is_empty() {
            self.results.clear();
            return;
        }

        self.searching = true;
        let query = self.query.clone();
        let root = root.to_path_buf();
        let opts = SearchOpts {
            regex: self.regex,
            case_sensitive: self.case_sensitive,
            whole_word: self.whole_word,
            include_glob: if self.include_glob.is_empty() {
                None
            } else {
                Some(self.include_glob.clone())
            },
            exclude_glob: if self.exclude_glob.is_empty() {
                None
            } else {
                Some(self.exclude_glob.clone())
            },
        };

        let tx = self.tx.clone();

        thread::spawn(move || {
            // Basic debounce simulation or just run
            // thread::sleep(Duration::from_millis(300));

            match ContentSearcher::search(&root, &query, opts) {
                Ok(results) => {
                    let _ = tx.send(results);
                }
                Err(e) => {
                    eprintln!("Search error: {}", e);
                    let _ = tx.send(Vec::new());
                }
            }
        });
    }

    pub fn update(&mut self) {
        if let Ok(results) = self.rx.try_recv() {
            self.results = results;
            self.searching = false;
        }
    }

    pub fn toggle_regex(&mut self) {
        self.regex = !self.regex;
    }
    pub fn toggle_case(&mut self) {
        self.case_sensitive = !self.case_sensitive;
    }
    pub fn toggle_word(&mut self) {
        self.whole_word = !self.whole_word;
    }
}
