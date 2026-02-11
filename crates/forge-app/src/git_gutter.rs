use anyhow::{Context, Result};
use git2::{DiffOptions, Repository};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DiffKind {
    Added,
    Modified,
    Deleted,
}

#[derive(Debug, Clone)]
pub struct GutterMark {
    pub line: usize, // 1-based
    pub kind: DiffKind,
}

pub struct GutterDiff;

impl GutterDiff {
    pub fn compute(repo_path: &Path, file_path: &Path) -> Result<Vec<GutterMark>> {
        let repo = Repository::open(repo_path).context("Failed to open git repo")?;

        // Compute diff between index and workdir for the specific file
        let mut opts = DiffOptions::new();
        opts.pathspec(file_path);

        // We need to handle potential errors if file is not in index (untracked)
        // If untracked, treat as all Added? Or none?
        // git diff usually shows nothing for untracked unless -u.
        // For gutter, we usually want to show modifications against HEAD or Index.
        // Usually against HEAD.
        // But instructions say `diff_index_to_workdir`.
        // This compares index (staged) to workdir.

        let diff = if let Ok(index) = repo.index() {
            repo.diff_index_to_workdir(Some(&index), Some(&mut opts))?
        } else {
            // Fallback?
            return Ok(Vec::new());
        };

        let mut marks = Vec::new();

        diff.foreach(
            &mut |_delta, _progress| true,
            None,
            None,
            Some(&mut |_delta, _hunk, line| {
                let kind = match line.origin() {
                    '+' => DiffKind::Added,
                    '-' => DiffKind::Deleted,
                    ' ' => return true, // Context, ignore
                    _ => return true,
                };

                // line.new_lineno() is Option<u32>. 1-based.
                // For Added/Modified, we use new_lineno.
                // For Deleted, it's attached to the line *before* or *after*?
                // Usually deletions are marked on the line where content was removed.
                // git2 provides old_lineno and new_lineno.

                if let Some(ln) = line.new_lineno() {
                    marks.push(GutterMark {
                        line: ln as usize,
                        kind,
                    });
                } else if kind == DiffKind::Deleted {
                    // Deleted lines don't have new_lineno.
                    // We might mark the previous line or next line.
                    // Common editor behavior: mark the line *after* the deletion with a triangle,
                    // or if at end, the last line.
                    // For now, let's just skip strictly visual representation of deleted lines
                    // or try to attach to old_lineno if it maps to something?
                    // But wait, old_lineno maps to the OLD file.
                    // The gutter is on the NEW file (workdir).
                    // So we need to map old_lineno to new_lineno context.
                    // This is hard without more context.
                    // VS Code puts a triangle on the line where deletion happened.
                    // That line has a new_lineno.
                    // But `line` struct in callback is for the deleted line itself.
                    // We need the hunk context?
                    // Actually, usually we process hunks.
                }
                true
            }),
        )?;

        Ok(marks)
    }
}
