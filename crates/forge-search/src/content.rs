//! Content search (grep) functionality using ripgrep primitives.

use anyhow::Result;
use grep::regex::RegexMatcherBuilder;
use grep::searcher::{BinaryDetection, SearcherBuilder, Sink, SinkMatch};
use ignore::WalkBuilder;
use std::path::Path;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct SearchOpts {
    pub regex: bool,
    pub case_sensitive: bool,
    pub whole_word: bool,
    pub include_glob: Option<String>,
    pub exclude_glob: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub file: String,
    pub line: usize,
    pub col: usize,
    pub text: String,
}

pub struct ContentSearcher;

// Custom sink to capture results with file context
struct CollectingSink<'a> {
    results: &'a mut Vec<SearchResult>,
    file_path: String,
}

impl<'a> Sink for CollectingSink<'a> {
    type Error = std::io::Error;

    fn matched(&mut self, _searcher: &grep::searcher::Searcher, mat: &SinkMatch) -> Result<bool, std::io::Error> {
        let text = std::str::from_utf8(mat.bytes()).unwrap_or("").trim().to_string();
        if text.len() > 1000 { return Ok(true); } // Skip long lines

        self.results.push(SearchResult {
            file: self.file_path.clone(),
            line: mat.line_number().unwrap_or(0) as usize,
            col: 0, // grep-searcher match doesn't give col easily without byte offset calc
            text,
        });
        Ok(true)
    }
}

impl ContentSearcher {
    pub fn search(root: &Path, query: &str, opts: SearchOpts) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();
        let results_mutex = Arc::new(Mutex::new(results));

        let mut builder = WalkBuilder::new(root);
        builder.hidden(false);

        if let Some(ref glob) = opts.include_glob {
            let mut overrides = ignore::overrides::OverrideBuilder::new(root);
            overrides.add(glob)?;
            builder.overrides(overrides.build()?);
        }
        if let Some(ref glob) = opts.exclude_glob {
            let mut overrides = ignore::overrides::OverrideBuilder::new(root);
            overrides.add(&format!("!{}", glob))?;
            builder.overrides(overrides.build()?);
        }

        // Prepare Matcher
        let mut matcher_builder = RegexMatcherBuilder::new();
        matcher_builder.case_insensitive(!opts.case_sensitive);
        if opts.whole_word {
            matcher_builder.word(true);
        }

        let pattern = if opts.regex {
            query.to_string()
        } else {
            regex::escape(query)
        };

        let matcher = matcher_builder.build(&pattern)?;

        // Parallel walk
        let walker = builder.build_parallel();

        walker.run(|| {
            let results_mutex = results_mutex.clone();
            let matcher = matcher.clone();

            Box::new(move |result| {
                use ignore::WalkState;

                let entry = match result {
                    Ok(entry) => entry,
                    Err(_) => return WalkState::Continue,
                };

                if let Some(ft) = entry.file_type() {
                    if !ft.is_file() {
                        return WalkState::Continue;
                    }
                } else {
                    return WalkState::Continue;
                }

                let path = entry.path();
                let path_string = path.to_string_lossy().to_string();

                let mut searcher = SearcherBuilder::new()
                    .binary_detection(BinaryDetection::quit(b'\x00'))
                    .line_number(true)
                    .build();

                let mut local_results = Vec::new();
                let mut sink = CollectingSink {
                    results: &mut local_results,
                    file_path: path_string,
                };

                let _ = searcher.search_path(&matcher, path, &mut sink);

                if !local_results.is_empty() {
                    if let Ok(mut lock) = results_mutex.lock() {
                        lock.extend(local_results);
                    }
                }

                WalkState::Continue
            })
        });

        // Extract results
        let final_results = Arc::try_unwrap(results_mutex)
            .map_err(|_| anyhow::anyhow!("Lock error"))?
            .into_inner()?;

        Ok(final_results)
    }
}
