//! Content search (grep) functionality.

use anyhow::Result;
use ignore::WalkBuilder;
use regex::RegexBuilder;
use std::fs;
use std::path::Path;

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

impl ContentSearcher {
    pub fn search(root: &Path, query: &str, opts: SearchOpts) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();

        let mut builder = WalkBuilder::new(root);
        // Default to respecting gitignore
        builder.hidden(false); // Search hidden files? Typically yes for .gitignore.
        // If hidden(true), it skips hidden files. We probably want to search hidden files if they are not ignored.
        // But typically .git is hidden and ignored.
        // Let's use defaults.

        if let Some(ref glob) = opts.include_glob {
            let mut overrides = ignore::overrides::OverrideBuilder::new(root);
            overrides.add(glob)?;
            builder.overrides(overrides.build()?);
        }
        if let Some(ref glob) = opts.exclude_glob {
             let mut overrides = ignore::overrides::OverrideBuilder::new(root);
            overrides.add(&format!("!{}", glob))?; // Exclude means ignore
            builder.overrides(overrides.build()?);
        }

        let walker = builder.build();

        // Prepare regex
        let mut pattern = if opts.regex {
            query.to_string()
        } else {
            regex::escape(query)
        };

        if opts.whole_word {
            pattern = format!(r"\b{}\b", pattern);
        }

        let re = RegexBuilder::new(&pattern)
            .case_insensitive(!opts.case_sensitive)
            .build()?;

        for result in walker {
            match result {
                Ok(entry) => {
                    if entry.file_type().is_some_and(|ft| ft.is_file()) {
                        let path = entry.path();
                        // Check if binary
                        // A simple heuristic: read first few bytes, if null byte, skip.
                        // Or use strict UTF-8 check.

                        // We will read the file as string. If it fails, it's likely binary.
                        // This is not efficient for huge files, but for a simple editor it's okay.
                        // A better way is memory mapping or reading line by line.
                        // For simplicity, read entire file.

                        match fs::read_to_string(path) {
                            Ok(content) => {
                                for (line_idx, line) in content.lines().enumerate() {
                                    if let Some(mat) = re.find(line) {
                                        results.push(SearchResult {
                                            file: path.to_string_lossy().to_string(),
                                            line: line_idx + 1,
                                            col: mat.start() + 1,
                                            text: line.trim().to_string(), // Trim for display
                                        });

                                        // TODO: Limit results per file?
                                    }
                                }
                            }
                            Err(_) => {
                                // Likely binary or permission error, skip
                            }
                        }
                    }
                }
                Err(err) => {
                    eprintln!("Error walking: {}", err);
                }
            }
        }

        Ok(results)
    }
}
