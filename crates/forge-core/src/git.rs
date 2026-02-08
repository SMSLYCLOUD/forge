use git2::{Repository, Status, StatusOptions};
use std::path::{Path, PathBuf};
use anyhow::Result;

pub struct GitIntegration {
    repo: Option<Repository>,
}

impl GitIntegration {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let repo = Repository::discover(path).ok();
        Self { repo }
    }

    pub fn status(&self, path: &Path) -> Option<FileStatus> {
        if let Some(repo) = &self.repo {
            // Check status of a specific file
            let status = repo.status_file(path).ok()?;

            if status.contains(Status::INDEX_NEW) || status.contains(Status::WT_NEW) {
                return Some(FileStatus::New);
            }
            if status.contains(Status::INDEX_MODIFIED) || status.contains(Status::WT_MODIFIED) {
                return Some(FileStatus::Modified);
            }
            if status.contains(Status::INDEX_DELETED) || status.contains(Status::WT_DELETED) {
                return Some(FileStatus::Deleted);
            }
             if status.contains(Status::IGNORED) {
                return Some(FileStatus::Ignored);
            }
        }
        None
    }

    pub fn branch_name(&self) -> Option<String> {
        if let Some(repo) = &self.repo {
            let head = repo.head().ok()?;
            return head.shorthand().map(|s| s.to_string());
        }
        None
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FileStatus {
    New,
    Modified,
    Deleted,
    Ignored,
}
