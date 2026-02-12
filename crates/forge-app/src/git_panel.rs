use anyhow::{Context, Result};
use git2::{IndexAddOption, Repository, Status, StatusOptions};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub enum FileStatus {
    Modified,
    Added,
    Deleted,
    Untracked,
    Renamed,
    Ignored,
    Conflicted,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct GitFile {
    pub path: String,
    pub status: FileStatus,
    pub staged: bool,
}

#[derive(Default)]
pub struct GitPanel {
    pub repo_path: Option<PathBuf>,
    pub files: Vec<GitFile>,
    // Repository is not Sync, so we might need to be careful if this struct is shared.
    // We'll re-open it when needed or keep it if we are single-threaded.
    // For now, we won't store Repository permanently to avoid Sync issues if any.
    // Or we store it and assume single-threaded usage.
    // Let's store it but wrap in a way or just re-open. Re-opening is safer for now.
}

impl GitPanel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn refresh(&mut self, repo_path: &Path) -> Result<()> {
        self.repo_path = Some(repo_path.to_path_buf());
        let repo = Repository::open(repo_path).context("Failed to open git repo")?;

        let mut opts = StatusOptions::new();
        opts.include_untracked(true);

        let statuses = repo
            .statuses(Some(&mut opts))
            .context("Failed to get statuses")?;

        self.files.clear();

        for entry in statuses.iter() {
            let status = entry.status();
            let path = entry.path().unwrap_or("").to_string();

            // Check staged status
            if status.contains(Status::INDEX_NEW)
                || status.contains(Status::INDEX_MODIFIED)
                || status.contains(Status::INDEX_DELETED)
                || status.contains(Status::INDEX_RENAMED)
                || status.contains(Status::INDEX_TYPECHANGE)
            {
                let kind = if status.contains(Status::INDEX_NEW) {
                    FileStatus::Added
                } else if status.contains(Status::INDEX_DELETED) {
                    FileStatus::Deleted
                } else if status.contains(Status::INDEX_RENAMED) {
                    FileStatus::Renamed
                } else {
                    FileStatus::Modified
                };

                self.files.push(GitFile {
                    path: path.clone(),
                    status: kind,
                    staged: true,
                });
            }

            // Check unstaged status
            if status.contains(Status::WT_NEW) {
                self.files.push(GitFile {
                    path: path.clone(),
                    status: FileStatus::Untracked,
                    staged: false,
                });
            } else if status.contains(Status::WT_MODIFIED) {
                self.files.push(GitFile {
                    path: path.clone(),
                    status: FileStatus::Modified,
                    staged: false,
                });
            } else if status.contains(Status::WT_DELETED) {
                self.files.push(GitFile {
                    path: path.clone(),
                    status: FileStatus::Deleted,
                    staged: false,
                });
            } else if status.contains(Status::WT_RENAMED) {
                self.files.push(GitFile {
                    path: path.clone(),
                    status: FileStatus::Renamed,
                    staged: false,
                });
            } else if status.contains(Status::CONFLICTED) {
                self.files.push(GitFile {
                    path: path.clone(),
                    status: FileStatus::Conflicted,
                    staged: false,
                });
            }
        }

        Ok(())
    }

    pub fn stage_file(&self, path: &str) -> Result<()> {
        if let Some(repo_path) = &self.repo_path {
            let repo = Repository::open(repo_path)?;
            let mut index = repo.index()?;
            index.add_path(Path::new(path))?;
            index.write()?;
        }
        Ok(())
    }

    pub fn unstage_file(&self, path: &str) -> Result<()> {
        if let Some(repo_path) = &self.repo_path {
            let repo = Repository::open(repo_path)?;
            let head = repo.head()?;

            // We want to reset index to HEAD for the file
            let obj = head.peel(git2::ObjectType::Commit)?;
            repo.reset_default(Some(&obj), [path].iter())?;
        }
        Ok(())
    }

    pub fn commit(&self, message: &str) -> Result<()> {
        if let Some(repo_path) = &self.repo_path {
            let repo = Repository::open(repo_path)?;
            let mut index = repo.index()?;
            let tree_id = index.write_tree()?;
            let tree = repo.find_tree(tree_id)?;

            let signature = repo.signature()?;
            let parent_commit = repo.head()?.peel_to_commit()?;

            repo.commit(
                Some("HEAD"),
                &signature,
                &signature,
                message,
                &tree,
                &[&parent_commit],
            )?;
        }
        Ok(())
    }
}
