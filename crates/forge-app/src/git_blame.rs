use anyhow::{Context, Result};
use git2::{BlameOptions, Repository};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct BlameLine {
    pub commit_hash: String,
    pub author: String,
    pub date: String,
    pub line_number: usize,
}

pub struct BlameView {
    pub lines: Vec<BlameLine>,
    pub visible: bool,
}

impl BlameView {
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            visible: false,
        }
    }

    pub fn compute(&mut self, repo_path: &Path, file_path: &Path) -> Result<()> {
        let repo = Repository::open(repo_path).context("Failed to open git repo")?;

        let mut opts = BlameOptions::new();
        // Maybe restrict range or other options?

        let blame = repo.blame_file(file_path, Some(&mut opts)).context("Failed to blame file")?;

        self.lines.clear();

        // Iterate through blame hunks
        for hunk in blame.iter() {
            let commit_id = hunk.final_commit_id();
            let commit = repo.find_commit(commit_id)?;
            let author = commit.author();
            let name = author.name().unwrap_or("Unknown").to_string();
            let time = commit.time();
            // Format time simply
            let date = format!("{}", time.seconds()); // Use chrono if available? standard just seconds.

            let start_line = hunk.final_start_line();
            let lines_in_hunk = hunk.lines_in_hunk();

            for i in 0..lines_in_hunk as usize {
                self.lines.push(BlameLine {
                    commit_hash: commit_id.to_string()[0..7].to_string(),
                    author: name.clone(),
                    date: date.clone(),
                    line_number: start_line + i,
                });
            }
        }

        Ok(())
    }
}

impl Default for BlameView {
    fn default() -> Self {
        Self::new()
    }
}
