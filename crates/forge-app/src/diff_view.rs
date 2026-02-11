use anyhow::{Context, Result};
use git2::{DiffOptions, Patch};

#[derive(Debug, Clone, PartialEq)]
pub enum DiffLineKind {
    Context,
    Added,
    Removed,
}

#[derive(Debug, Clone)]
pub struct DiffLine {
    pub kind: DiffLineKind,
    pub text: String,
    pub old_lineno: Option<u32>,
    pub new_lineno: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct DiffHunk {
    pub old_start: u32,
    pub old_lines: u32,
    pub new_start: u32,
    pub new_lines: u32,
    pub lines: Vec<DiffLine>,
}

pub struct DiffView {
    pub hunks: Vec<DiffHunk>,
}

impl DiffView {
    pub fn new() -> Self {
        Self { hunks: Vec::new() }
    }

    pub fn compute_diff(old: &str, new: &str) -> Result<Vec<DiffHunk>> {
        let mut opts = DiffOptions::new();
        opts.context_lines(3);

        let patch = Patch::from_buffers(
            old.as_bytes(),
            None,
            new.as_bytes(),
            None,
            Some(&mut opts),
        ).context("Failed to compute diff")?;

        let mut hunks = Vec::new();

        for i in 0..patch.num_hunks() {
            let (hunk, lines_count) = patch.hunk(i)?;

            let mut diff_hunk = DiffHunk {
                old_start: hunk.old_start(),
                old_lines: hunk.old_lines(),
                new_start: hunk.new_start(),
                new_lines: hunk.new_lines(),
                lines: Vec::new(),
            };

            for j in 0..lines_count {
                let line = patch.line_in_hunk(i, j)?;
                let kind = match line.origin() {
                    '+' => DiffLineKind::Added,
                    '-' => DiffLineKind::Removed,
                    ' ' => DiffLineKind::Context,
                    _ => DiffLineKind::Context,
                };

                let text = std::str::from_utf8(line.content()).unwrap_or("").trim_end().to_string();

                diff_hunk.lines.push(DiffLine {
                    kind,
                    text,
                    old_lineno: line.old_lineno(),
                    new_lineno: line.new_lineno(),
                });
            }
            hunks.push(diff_hunk);
        }

        Ok(hunks)
    }
}

impl Default for DiffView {
    fn default() -> Self {
        Self::new()
    }
}
